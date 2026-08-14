use crate::middleware::permission::PermissionContext;
use crate::repository::dictionary_repository::DictionaryRepository;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use common::entity::dictionary::Dictionary;
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::entity::user::User;
use common::models::ApiResponse;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListParams {
    category: Option<String>,
}

/// List dictionary items
pub async fn list_dictionaries(
    Extension(repo): Extension<Arc<DictionaryRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let result = if let Some(category) = params.category {
        repo.list_by_category(&category).await
    } else {
        repo.list_all().await
    };

    match result {
        Ok(items) => {
            let scope = perm_ctx
                .evaluate(&ResourceType::Dictionary, &PermissionAction::View)
                .unwrap_or(ScopeConstraint::None);
            let items = PermissionContext::filter_by_scope(items, &scope, &perm_ctx.user_id, |d| {
                d.created_by.as_deref()
            });
            let response = ApiResponse {
                status: 200,
                message: "Success".to_string(),
                data: Some(items),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Vec<Dictionary>> {
                status: 500,
                message: format!("Failed to list dictionary items: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get a dictionary item by ID
pub async fn get_dictionary(
    Path(id): Path<String>,
    Extension(repo): Extension<Arc<DictionaryRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match repo.get(&id).await {
        Ok(Some(item)) => {
            if !perm_ctx.allows_resource(
                &ResourceType::Dictionary,
                &PermissionAction::View,
                item.created_by.as_deref(),
            ) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<Dictionary> {
                        status: 403,
                        message: "Forbidden".into(),
                        data: None,
                    }),
                )
                    .into_response();
            }
            let response = ApiResponse {
                status: 200,
                message: "Success".to_string(),
                data: Some(item),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::<Dictionary> {
                status: 404,
                message: "Dictionary item not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Dictionary> {
                status: 500,
                message: format!("Failed to get dictionary item: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Create a new dictionary item
pub async fn create_dictionary(
    Extension(repo): Extension<Arc<DictionaryRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(user): Extension<User>,
    Json(mut item): Json<Dictionary>,
) -> impl IntoResponse {
    if !perm_ctx.allows_action(&ResourceType::Dictionary, &PermissionAction::Create) {
        let response = ApiResponse::<Dictionary> {
            status: 403,
            message: "Forbidden: insufficient permission to create dictionary items".to_string(),
            data: None,
        };
        return (StatusCode::FORBIDDEN, Json(response)).into_response();
    }

    // Ensure ID is set
    if item.id.is_empty() {
        item.id = Uuid::new_v4().to_string();
    }

    // Set timestamps and creator
    let now = Utc::now().to_rfc3339();
    item.created_at = now.clone();
    item.updated_at = now;
    item.created_by = Some(user.id);

    match repo.save(&item).await {
        Ok(_) => {
            let response = ApiResponse {
                status: 201,
                message: "Dictionary item created successfully".to_string(),
                data: Some(item),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Dictionary> {
                status: 500,
                message: format!("Failed to create dictionary item: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Update a dictionary item
pub async fn update_dictionary(
    Path(id): Path<String>,
    Extension(repo): Extension<Arc<DictionaryRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(mut item): Json<Dictionary>,
) -> impl IntoResponse {
    // Check if exists
    match repo.get(&id).await {
        Ok(Some(existing_item)) => {
            if !perm_ctx.allows_resource(
                &ResourceType::Dictionary,
                &PermissionAction::Update,
                existing_item.created_by.as_deref(),
            ) {
                let response = ApiResponse::<Dictionary> {
                    status: 403,
                    message: "Forbidden: insufficient permission to update this dictionary item"
                        .to_string(),
                    data: None,
                };
                return (StatusCode::FORBIDDEN, Json(response)).into_response();
            }

            // Preserve creation time, ID, and created_by
            let created_by = existing_item.created_by.clone();
            item.id = id;
            item.created_at = existing_item.created_at;
            item.created_by = created_by;
            item.updated_at = Utc::now().to_rfc3339();

            match repo.save(&item).await {
                Ok(_) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Dictionary item updated successfully".to_string(),
                        data: Some(item),
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => {
                    let response = ApiResponse::<Dictionary> {
                        status: 500,
                        message: format!("Failed to update dictionary item: {}", e),
                        data: None,
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<Dictionary> {
                status: 404,
                message: "Dictionary item not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Dictionary> {
                status: 500,
                message: format!("Failed to check dictionary item existence: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Delete a dictionary item
pub async fn delete_dictionary(
    Path(id): Path<String>,
    Extension(repo): Extension<Arc<DictionaryRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match repo.get(&id).await {
        Ok(Some(item)) => {
            if !perm_ctx.allows_resource(
                &ResourceType::Dictionary,
                &PermissionAction::Delete,
                item.created_by.as_deref(),
            ) {
                let response = ApiResponse::<()> {
                    status: 403,
                    message: "Forbidden: insufficient permission to delete this dictionary item"
                        .to_string(),
                    data: None,
                };
                return (StatusCode::FORBIDDEN, Json(response)).into_response();
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()> {
                status: 404,
                message: "Dictionary item not found".to_string(),
                data: None,
            };
            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to get dictionary item: {}", e),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    }

    match repo.delete(&id).await {
        Ok(_) => {
            let response = ApiResponse::<()> {
                status: 200,
                message: "Dictionary item deleted successfully".to_string(),
                data: None,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to delete dictionary item: {}", e),
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

    async fn make_post(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    async fn make_get(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::GET).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    async fn make_put(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
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
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    async fn make_delete(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::DELETE).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn test_create_dictionary() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(
            &app.router,
            "/api/v1/dictionaries",
            Some(&app.admin_token),
            json!({
                "key": "Linux",
                "category": "OS",
                "value": "linux"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["status"], 201);
        assert_eq!(body["data"]["key"], "Linux");
    }

    #[tokio::test]
    async fn test_list_dictionaries() {
        let app = TestAppBuilder::new().build().await;
        let _ = make_post(
            &app.router,
            "/api/v1/dictionaries",
            Some(&app.admin_token),
            json!({
                "key": "Linux",
                "category": "OS",
                "value": "linux"
            }),
        )
        .await;
        let (status, body) =
            make_get(&app.router, "/api/v1/dictionaries", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }

    #[tokio::test]
    async fn test_get_dictionary_not_found() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(
            &app.router,
            "/api/v1/dictionaries/nonexistent",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["status"], 404);
    }

    #[tokio::test]
    async fn test_get_dictionary() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/dictionaries",
            Some(&app.admin_token),
            json!({
                "key": "Linux",
                "category": "OS",
                "value": "linux"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let id = create_body["data"]["id"].as_str().unwrap();
        let (status, body) = make_get(
            &app.router,
            &format!("/api/v1/dictionaries/{}", id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
        assert_eq!(body["data"]["key"], "Linux");
    }

    #[tokio::test]
    async fn test_update_dictionary() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/dictionaries",
            Some(&app.admin_token),
            json!({
                "key": "Linux",
                "category": "OS",
                "value": "linux"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let id = create_body["data"]["id"].as_str().unwrap();
        let (status, body) = make_put(
            &app.router,
            &format!("/api/v1/dictionaries/{}", id),
            Some(&app.admin_token),
            json!({
                "key": "Updated",
                "category": "OS",
                "value": "updated"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }

    #[tokio::test]
    async fn test_delete_dictionary() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/dictionaries",
            Some(&app.admin_token),
            json!({
                "key": "Linux",
                "category": "OS",
                "value": "linux"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let id = create_body["data"]["id"].as_str().unwrap();
        let (status, body) = make_delete(
            &app.router,
            &format!("/api/v1/dictionaries/{}", id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }
}
