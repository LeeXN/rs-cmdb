use crate::queue::{Message, MessageQueue};
use crate::repository::user_repository::UserRepository;
use crate::service::auth_service::{AuthService, validate_password_complexity};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use common::command::{AuditAction, AuditLogEntry};
use common::entity::user::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, RefreshRequest, Role, User, UserResponse,
};
use common::models::ApiResponse;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub async fn change_password(
    Extension(user_repo): Extension<Arc<UserRepository>>,
    Extension(auth_service): Extension<Arc<AuthService>>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Validate new password complexity
    if let Err(e) = validate_password_complexity(&payload.new_password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                status: 400,
                message: e.log_and_user_message(),
                data: None,
            }),
        );
    }

    // Verify old password
    match auth_service.verify_password(&payload.old_password, &current_user.password_hash) {
        Ok(true) => {}
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()> {
                    status: 400,
                    message: "Invalid old password".to_string(),
                    data: None,
                }),
            );
        }
    }

    // Hash new password
    let new_hash = match auth_service.hash_password(&payload.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Password hashing error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: 500,
                    message: "Failed to process password".to_string(),
                    data: None,
                }),
            );
        }
    };

    // Update user
    let mut updated_user = current_user.clone();
    updated_user.password_hash = new_hash;

    match user_repo.save(&updated_user).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "Password changed successfully".to_string(),
                data: None,
            }),
        ),
        Err(e) => {
            error!("Failed to update user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: 500,
                    message: "Failed to update user".to_string(),
                    data: None,
                }),
            )
        }
    }
}

pub async fn login(
    Extension(user_repo): Extension<Arc<UserRepository>>,
    Extension(auth_service): Extension<Arc<AuthService>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match user_repo.find_by_username(&payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<LoginResponse> {
                    status: 401,
                    message: "Invalid username or password".to_string(),
                    data: None,
                }),
            );
        }
        Err(e) => {
            error!("Database error during login: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LoginResponse> {
                    status: 500,
                    message: "Internal server error".to_string(),
                    data: None,
                }),
            );
        }
    };

    if !user.is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<LoginResponse> {
                status: 403,
                message: "User account is inactive".to_string(),
                data: None,
            }),
        );
    }

    match auth_service.verify_password(&payload.password, &user.password_hash) {
        Ok(true) => {
            match auth_service.generate_access_refresh_pair(&user) {
                Ok((token, refresh_token, _access_exp, _refresh_exp)) => {
                    // Update last login
                    let mut updated_user = user.clone();
                    updated_user.last_login = Some(Utc::now().to_rfc3339());
                    if let Err(e) = user_repo.save(&updated_user).await {
                        error!("Failed to update last login: {}", e);
                    }

                    (
                        StatusCode::OK,
                        Json(ApiResponse {
                            status: 200,
                            message: "Login successful".to_string(),
                            data: Some(LoginResponse {
                                token,
                                refresh_token,
                                user: updated_user.into(),
                            }),
                        }),
                    )
                }
                Err(e) => {
                    error!("Token generation error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<LoginResponse> {
                            status: 500,
                            message: "Failed to generate token".to_string(),
                            data: None,
                        }),
                    )
                }
            }
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<LoginResponse> {
                status: 401,
                message: "Invalid username or password".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn register(
    Extension(user_repo): Extension<Arc<UserRepository>>,
    Extension(auth_service): Extension<Arc<AuthService>>,
    Extension(operator): Extension<User>,
    Extension(message_queue): Extension<Arc<dyn MessageQueue>>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Validate password complexity
    if let Err(e) = validate_password_complexity(&payload.password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse> {
                status: 400,
                message: e.log_and_user_message(),
                data: None,
            }),
        );
    }

    // Check if user exists
    match user_repo.find_by_username(&payload.username).await {
        Ok(Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UserResponse> {
                    status: 400,
                    message: "Username already exists".to_string(),
                    data: None,
                }),
            );
        }
        Err(e) => {
            error!("Database error during registration: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse> {
                    status: 500,
                    message: "Internal server error".to_string(),
                    data: None,
                }),
            );
        }
        Ok(None) => {}
    }

    let password_hash = match auth_service.hash_password(&payload.password) {
        Ok(hash) => hash,
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
    };

    let username = payload.username.clone();
    let user = User {
        id: Uuid::new_v4().to_string(),
        username,
        password_hash,
        role: payload.role.unwrap_or(Role::Viewer),
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    match user_repo.save(&user).await {
        Ok(_) => {
            let audit = AuditLogEntry::new(AuditAction::UserCreated, &operator.username, &format!("Created user {}", user.username));
            let _ = message_queue.send_message(Message::AuditLog(audit));
            (
                StatusCode::CREATED,
                Json(ApiResponse {
                    status: 201,
                    message: "User registered successfully".to_string(),
                    data: Some(user.into()),
                }),
            )
        }
        Err(e) => {
            error!("Failed to save user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse> {
                    status: 500,
                    message: "Failed to create user".to_string(),
                    data: None,
                }),
            )
        }
    }
}

pub async fn me(Extension(current_user): Extension<User>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse {
            status: 200,
            message: "Current user info".to_string(),
            data: Some(UserResponse::from(current_user)),
        }),
    )
}

pub async fn refresh(
    Extension(auth_service): Extension<Arc<AuthService>>,
    Extension(user_repo): Extension<Arc<UserRepository>>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    // Verify refresh token
    let claims = match auth_service.verify_refresh_token(&payload.refresh_token) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<LoginResponse> {
                    status: 401,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            );
        }
    };

    // Check blacklist
    if auth_service.check_blacklist(&claims.jti).await.is_err() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<LoginResponse> {
                status: 401,
                message: "Refresh token has been revoked".to_string(),
                data: None,
            }),
        );
    }

    // Revoke the old refresh token (rotation)
    auth_service.revoke_token(&claims.jti, claims.exp).await;

    // Fetch user to generate new token pair
    let user = match user_repo.get(&claims.sub).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<LoginResponse> {
                    status: 401,
                    message: "User not found".to_string(),
                    data: None,
                }),
            );
        }
        Err(e) => {
            error!("Database error during token refresh: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LoginResponse> {
                    status: 500,
                    message: "Internal server error".to_string(),
                    data: None,
                }),
            );
        }
    };

    if !user.is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<LoginResponse> {
                status: 403,
                message: "User account is inactive".to_string(),
                data: None,
            }),
        );
    }

    match auth_service.generate_access_refresh_pair(&user) {
        Ok((token, refresh_token, _access_exp, _refresh_exp)) => {
            (
                StatusCode::OK,
                Json(ApiResponse {
                    status: 200,
                    message: "Token refreshed successfully".to_string(),
                    data: Some(LoginResponse {
                        token,
                        refresh_token,
                        user: user.into(),
                    }),
                }),
            )
        }
        Err(e) => {
            error!("Token generation error during refresh: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LoginResponse> {
                    status: 500,
                    message: "Failed to generate tokens".to_string(),
                    data: None,
                }),
            )
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

    async fn make_post(app: &axum::Router, path: &str, token: Option<&str>, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::from(serde_json::to_vec(&body).unwrap())).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        (status, body)
    }

    async fn make_get(app: &axum::Router, path: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::GET)
            .uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn test_login_valid_credentials() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(&app.router, "/api/v1/auth/login", None, json!({
            "username": "test_admin",
            "password": "admin123"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
        assert!(body["data"]["token"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(&app.router, "/api/v1/auth/login", None, json!({
            "username": "test_admin",
            "password": "wrong_password"
        })).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["status"], 401);
    }

    #[tokio::test]
    async fn test_me_without_token_returns_401() {
        let app = TestAppBuilder::new().build().await;
        let mut req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/auth/me")
            .body(Body::empty()).unwrap();
        let resp = app.router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_me_with_valid_token() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(&app.router, "/api/v1/auth/me", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["username"], "test_admin");
    }

    #[tokio::test]
    async fn test_register_valid_user() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(&app.router, "/api/v1/auth/register", Some(&app.admin_token), json!({
            "username": "new_user",
            "password": "ValidPass123!"
        })).await;
        assert_eq!(status, StatusCode::CREATED, "body: {:?}", body);
        assert_eq!(body["data"]["username"], "new_user");
    }

    #[tokio::test]
    async fn test_register_requires_admin() {
        let app = TestAppBuilder::new().build().await;
        let mut req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/register")
            .header(header::CONTENT_TYPE, "application/json");
        let (k, v) = auth_headers(&app.auth_token);
        req = req.header(k, v);
        let req = req.body(Body::from(serde_json::to_vec(&json!({
            "username": "hacker",
            "password": "ValidPass123!"
        })).unwrap())).unwrap();
        let resp = app.router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let app = TestAppBuilder::new().build().await;
        let (status, _) = make_post(&app.router, "/api/v1/auth/register", Some(&app.admin_token), json!({
            "username": "test_admin",
            "password": "ValidPass123!"
        })).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_weak_password() {
        let app = TestAppBuilder::new().build().await;
        let (status, _) = make_post(&app.router, "/api/v1/auth/register", Some(&app.admin_token), json!({
            "username": "another_user",
            "password": "weak"
        })).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
