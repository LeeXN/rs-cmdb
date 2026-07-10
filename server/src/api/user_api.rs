use crate::queue::{Message, MessageQueue};
use crate::repository::user_repository::UserRepository;
use crate::service::auth_service::AuthService;
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use common::command::{AuditAction, AuditLogEntry};
use common::entity::user::{UpdateUserRequest, UserResponse};
use common::entity::user::User;
use common::models::ApiResponse;
use std::sync::Arc;
use tracing::{error, info};

/// List all users
pub async fn list_users(Extension(user_repo): Extension<Arc<UserRepository>>) -> impl IntoResponse {
    match user_repo.list_all().await {
        Ok(users) => {
            let user_responses: Vec<UserResponse> =
                users.into_iter().map(UserResponse::from).collect();
            let response = ApiResponse {
                status: 200,
                message: "Success".to_string(),
                data: Some(user_responses),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to list users: {}", e);
            let response = ApiResponse::<Vec<UserResponse>> {
                status: 500,
                message: format!("Failed to list users: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Update a user
pub async fn update_user(
    Path(id): Path<String>,
    Extension(user_repo): Extension<Arc<UserRepository>>,
    Extension(auth_service): Extension<Arc<AuthService>>,
    Extension(operator): Extension<User>,
    Extension(message_queue): Extension<Arc<dyn MessageQueue>>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    // Find user
    let mut user = match user_repo.get(&id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<UserResponse> {
                    status: 404,
                    message: "User not found".to_string(),
                    data: None,
                }),
            );
        }
        Err(e) => {
            error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse> {
                    status: 500,
                    message: "Internal server error".to_string(),
                    data: None,
                }),
            );
        }
    };

    let old_role = user.role.clone();

    // Update fields
    if let Some(role) = payload.role {
        user.role = role;
    }
    if let Some(is_active) = payload.is_active {
        user.is_active = is_active;
    }
    if let Some(password) = payload.password {
        match auth_service.hash_password(&password) {
            Ok(hash) => user.password_hash = hash,
            Err(e) => {
                error!("Password hashing error: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<UserResponse> {
                        status: 500,
                        message: "Failed to process password".to_string(),
                        data: None,
                    }),
                );
            }
        }
    }

    match user_repo.save(&user).await {
        Ok(_) => {
            if user.role != old_role {
                let audit = AuditLogEntry::new(AuditAction::UserRoleChanged, &operator.username, &format!("User {} role changed from {:?} to {:?}", user.username, old_role, user.role));
                let _ = message_queue.send_message(Message::AuditLog(audit));
            }
            (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "User updated successfully".to_string(),
                data: Some(user.into()),
            }),
        )},
        Err(e) => {
            error!("Failed to save user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse> {
                    status: 500,
                    message: "Failed to update user".to_string(),
                    data: None,
                }),
            )
        }
    }
}

/// Delete a user
pub async fn delete_user(
    Path(id): Path<String>,
    Extension(user_repo): Extension<Arc<UserRepository>>,
    Extension(operator): Extension<User>,
    Extension(message_queue): Extension<Arc<dyn MessageQueue>>,
) -> impl IntoResponse {
    info!("Deleting user: {}", id);
    match user_repo.delete(&id).await {
        Ok(_) => {
            let audit = AuditLogEntry::new(AuditAction::UserDeleted, &operator.username, &format!("Deleted user {}", id));
            let _ = message_queue.send_message(Message::AuditLog(audit));
            let response = ApiResponse::<()> {
                status: 200,
                message: "User deleted successfully".to_string(),
                data: None,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to delete user {}: {}", id, e);
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to delete user: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::{TestAppBuilder, auth_headers};
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode, header},
    };
    use serde_json::json;
    use tower::ServiceExt;

    async fn make_get(app: &axum::Router, path: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::GET).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    async fn make_delete(app: &axum::Router, path: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::DELETE).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    async fn make_put(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body_val: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::PUT)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req
            .body(Body::from(serde_json::to_vec(&body_val).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    #[tokio::test]
    async fn test_list_users() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) =
            make_get(&app.router, "/api/v1/accounts", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        let data = body["data"].as_array().unwrap();
        assert!(data.len() >= 2);
    }

    #[tokio::test]
    async fn test_list_users_non_admin() {
        let app = TestAppBuilder::new().build().await;
        let (status, _body) =
            make_get(&app.router, "/api/v1/accounts", Some(&app.auth_token)).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let app = TestAppBuilder::new().build().await;
        let user_id = &app.test_user.id;
        let (status, _body) = make_delete(
            &app.router,
            &format!("/api/v1/accounts/{}", user_id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_user() {
        let app = TestAppBuilder::new().build().await;
        let user_id = &app.test_user.id;
        let (status, body) = make_put(
            &app.router,
            &format!("/api/v1/accounts/{}", user_id),
            Some(&app.admin_token),
            json!({"role": "Viewer"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["role"], "Viewer");
    }
}
