use crate::repository::client_repository::ClientRepository;
use crate::middleware::permission::PermissionContext;
use crate::repository::rack_repository::RackRepository;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use chrono::Utc;
use common::entity::user::User;
use common::models::{ApiResponse, PaginatedResult, Rack, RackQuery};
use std::sync::Arc;
use uuid::Uuid;

/// List all racks
pub async fn list_racks(
    Query(query): Query<RackQuery>,
    Extension(rack_repo): Extension<Arc<RackRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match rack_repo.list_all().await {
        Ok(mut racks) => {
            let scope = perm_ctx
                .evaluate(&ResourceType::Rack, &PermissionAction::View)
                .unwrap_or(ScopeConstraint::None);
            racks = PermissionContext::filter_by_scope(
                racks,
                &scope,
                &perm_ctx.user_id,
                |r| r.created_by.as_deref(),
            );
            // Filter by search term
            if let Some(ref search) = query.search {
                let search_lower = search.to_lowercase();
                racks.retain(|r| {
                    r.name.to_lowercase().contains(&search_lower)
                        || r.location
                            .as_ref()
                            .is_some_and(|l| l.to_lowercase().contains(&search_lower))
                        || r.description
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&search_lower))
                });
            }

            // Filter by location
            if let Some(ref location) = query.location
                && !location.is_empty()
            {
                racks.retain(|r| r.location.as_ref() == Some(location));
            }

            // Sort by name
            racks.sort_by(|a, b| a.name.cmp(&b.name));

            // Pagination
            let total = racks.len();
            let page = query.page.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            let total_pages = (total as f64 / page_size as f64).ceil() as usize;

            let start = (page - 1) * page_size;
            let end = std::cmp::min(start + page_size, total);

            let items = if start < total {
                racks[start..end].to_vec()
            } else {
                Vec::new()
            };

            let result = PaginatedResult {
                items,
                total,
                page,
                page_size,
                total_pages,
            };

            let response = ApiResponse {
                status: 200,
                message: "Success".to_string(),
                data: Some(result),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<PaginatedResult<Rack>> {
                status: 500,
                message: format!("Failed to list racks: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get a rack by ID
pub async fn get_rack(
    Path(id): Path<String>,
    Extension(rack_repo): Extension<Arc<RackRepository>>,
) -> impl IntoResponse {
    match rack_repo.get(&id).await {
        Ok(Some(rack)) => {
            let response = ApiResponse {
                status: 200,
                message: "Success".to_string(),
                data: Some(rack),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::<()> {
                status: 404,
                message: "Rack not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to get rack: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Create a new rack
pub async fn create_rack(
    Extension(user): Extension<User>,
    Extension(rack_repo): Extension<Arc<RackRepository>>,
    Json(mut rack): Json<Rack>,
) -> impl IntoResponse {
    // Ensure ID is set
    if rack.id.is_empty() {
        rack.id = Uuid::new_v4().to_string();
    }

    // Set timestamps and ownership
    let now = Utc::now().to_rfc3339();
    rack.created_by = Some(user.id);
    rack.created_at = now.clone();
    rack.updated_at = now;

    match rack_repo.save(&rack).await {
        Ok(_) => {
            let response = ApiResponse {
                status: 201,
                message: "Rack created successfully".to_string(),
                data: Some(rack),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to create rack: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Update a rack
pub async fn update_rack(
    Path(id): Path<String>,
    Extension(current_user): Extension<User>,
    Extension(rack_repo): Extension<Arc<RackRepository>>,
    Json(mut rack): Json<Rack>,
) -> impl IntoResponse {
    // Check if rack exists
    match rack_repo.get(&id).await {
        Ok(Some(existing)) => {
            // Ownership check
            if current_user.role != common::entity::user::Role::Admin
                && existing.created_by.as_deref() != Some(&current_user.id)
            {
                let response = ApiResponse::<()> {
                    status: 403,
                    message: "Forbidden: you can only update resources you created".to_string(),
                    data: None,
                };
                return (StatusCode::FORBIDDEN, Json(response)).into_response();
            }

            // Preserve ownership and created_at
            rack.created_by = existing.created_by;
            rack.created_at = existing.created_at;
            rack.id = id;
            rack.updated_at = Utc::now().to_rfc3339();

            match rack_repo.save(&rack).await {
                Ok(_) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Rack updated successfully".to_string(),
                        data: Some(rack),
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => {
                    let response = ApiResponse::<()> {
                        status: 500,
                        message: format!("Failed to update rack: {}", e),
                        data: None,
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()> {
                status: 404,
                message: "Rack not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to get rack: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Delete a rack
pub async fn delete_rack(
    Path(id): Path<String>,
    Extension(current_user): Extension<User>,
    Extension(rack_repo): Extension<Arc<RackRepository>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    // Ownership check
    match rack_repo.get(&id).await {
        Ok(Some(existing)) => {
            if current_user.role != common::entity::user::Role::Admin
                && existing.created_by.as_deref() != Some(&current_user.id)
            {
                let response = ApiResponse::<()> {
                    status: 403,
                    message: "Forbidden: you can only delete resources you created".to_string(),
                    data: None,
                };
                return (StatusCode::FORBIDDEN, Json(response)).into_response();
            }
        }
        _ => {}
    }

    // Check if any clients are using this rack
    match client_repo.count_by_rack(&id).await {
        Ok(count) if count > 0 => {
            let response = ApiResponse::<()> {
                status: 400,
                message: format!(
                    "Cannot delete rack: {} clients are still assigned to it",
                    count
                ),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to check rack usage: {}", e),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
        _ => {}
    }

    match rack_repo.delete(&id).await {
        Ok(_) => {
            let response = ApiResponse::<()> {
                status: 200,
                message: "Rack deleted successfully".to_string(),
                data: None,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to delete rack: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

// Helper implementation for ApiResponse to make it easier to construct responses
// impl<T: PartialEq> ApiResponse<T> {
//     pub fn new_success(message: String) -> Self {
//         Self {
//             status: 200,
//             message,
//             data: None,
//         }
//     }

//     pub fn new_error(status: u16, message: String) -> Self {
//         Self {
//             status,
//             message,
//             data: None,
//         }
// }
// }

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

    async fn make_delete(app: &axum::Router, path: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::DELETE)
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
    async fn test_create_rack() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(&app.router, "/api/v1/racks", Some(&app.admin_token), json!({
            "name": "RACK-A1",
            "location": "DC1",
            "height_u": 42
        })).await;
        assert_eq!(status, StatusCode::CREATED, "body: {:?}", body);
        assert_eq!(body["data"]["name"], "RACK-A1");
        assert_eq!(body["data"]["height_u"], 42);
    }

    #[tokio::test]
    async fn test_list_racks() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, _) = make_post(&app.router, "/api/v1/racks", Some(&app.admin_token), json!({
            "name": "RACK-LIST",
            "height_u": 42
        })).await;
        assert_eq!(create_status, StatusCode::CREATED);

        let (status, body) = make_get(&app.router, "/api/v1/racks", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert!(body["data"]["total"].as_u64().unwrap_or(0) >= 1);
    }

    #[tokio::test]
    async fn test_get_rack() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(&app.router, "/api/v1/racks", Some(&app.admin_token), json!({
            "name": "RACK-GET",
            "height_u": 42
        })).await;
        assert_eq!(create_status, StatusCode::CREATED);
        let rack_id = create_body["data"]["id"].as_str().unwrap().to_string();

        let (status, body) = make_get(&app.router, &format!("/api/v1/racks/{}", rack_id), Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert_eq!(body["data"]["name"], "RACK-GET");
    }

    #[tokio::test]
    async fn test_create_rack_duplicate() {
        let app = TestAppBuilder::new().build().await;
        let (status1, body1) = make_post(&app.router, "/api/v1/racks", Some(&app.admin_token), json!({
            "name": "RACK-A1",
            "location": "DC1",
            "height_u": 42
        })).await;
        assert_eq!(status1, StatusCode::CREATED, "body: {:?}", body1);

        // The handler does not enforce unique names; a second rack with the same
        // name gets a new UUID and should also succeed.
        let (status2, body2) = make_post(&app.router, "/api/v1/racks", Some(&app.admin_token), json!({
            "name": "RACK-A1",
            "location": "DC1",
            "height_u": 42
        })).await;
        assert_eq!(status2, StatusCode::CREATED, "body: {:?}", body2);
    }
}
