use crate::middleware::permission::PermissionContext;
use crate::repository::client_repository::ClientRepository;
use crate::repository::project_repository::ProjectRepository;
use crate::service::validation_service::ValidationService;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::entity::user::User;
use common::models::{ApiResponse, PaginatedResult, Project, ProjectQuery};
use std::sync::Arc;
use uuid::Uuid;

/// List all projects
pub async fn list_projects(
    Query(query): Query<ProjectQuery>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match project_repo.list_all().await {
        Ok(mut projects) => {
            let scope = perm_ctx
                .evaluate(&ResourceType::Project, &PermissionAction::View)
                .unwrap_or(ScopeConstraint::None);
            projects.retain(|project| {
                PermissionContext::matches_scope(
                    &scope,
                    &perm_ctx.user_id,
                    project.created_by.as_deref(),
                    Some(project.id.as_str()),
                    &[],
                )
            });
            // Filter by search term
            if let Some(ref search) = query.search {
                let search_lower = search.to_lowercase();
                projects.retain(|p| {
                    p.name.to_lowercase().contains(&search_lower)
                        || p.code
                            .as_ref()
                            .is_some_and(|c| c.to_lowercase().contains(&search_lower))
                        || p.department
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&search_lower))
                });
            }

            // Filter by department
            if let Some(ref department) = query.department
                && !department.is_empty()
            {
                projects.retain(|p| p.department.as_ref() == Some(department));
            }

            // Sort by name
            projects.sort_by(|a, b| a.name.cmp(&b.name));

            // Pagination
            let total = projects.len();
            let page = query.page.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            let total_pages = (total as f64 / page_size as f64).ceil() as usize;

            let start = (page - 1) * page_size;
            let end = std::cmp::min(start + page_size, total);

            let items = if start < total {
                projects[start..end].to_vec()
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
            let response = ApiResponse::<PaginatedResult<Project>> {
                status: 500,
                message: format!("Failed to list projects: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get a project by ID
pub async fn get_project(
    Path(id): Path<String>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match project_repo.get(&id).await {
        Ok(Some(project)) => {
            if !perm_ctx.allows_resource_with_scope(
                &ResourceType::Project,
                &PermissionAction::View,
                project.created_by.as_deref(),
                Some(project.id.as_str()),
                &[],
            ) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<Project> {
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
                data: Some(project),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::<Project> {
                status: 404,
                message: "Project not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Project> {
                status: 500,
                message: format!("Failed to get project: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Create a new project
pub async fn create_project(
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(user): Extension<User>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Json(mut project): Json<Project>,
) -> impl IntoResponse {
    // Validate manager_id
    if let Some(manager_id) = &project.manager_id
        && !manager_id.is_empty()
        && let Err(e) = validation_service.validate_person_exists(manager_id).await
    {
        let response = ApiResponse::<Project> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        )
            .into_response();
    }

    // Ensure ID is set
    if project.id.is_empty() {
        project.id = Uuid::new_v4().to_string();
    }

    if !perm_ctx.allows_action_with_scope(
        &ResourceType::Project,
        &PermissionAction::Create,
        Some(project.id.as_str()),
        &[],
    ) {
        let response = ApiResponse::<Project> {
            status: 403,
            message: "Forbidden: insufficient permission to create projects".to_string(),
            data: None,
        };
        return (StatusCode::FORBIDDEN, Json(response)).into_response();
    }

    // Set ownership and timestamps
    let now = Utc::now().to_rfc3339();
    project.created_by = Some(user.id);
    project.created_at = now.clone();
    project.updated_at = now;

    match project_repo.save(&project).await {
        Ok(_) => {
            let response = ApiResponse {
                status: 201,
                message: "Project created successfully".to_string(),
                data: Some(project),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Project> {
                status: 500,
                message: format!("Failed to create project: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Update a project
pub async fn update_project(
    Path(id): Path<String>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Json(mut project): Json<Project>,
) -> impl IntoResponse {
    // Validate manager_id
    if let Some(manager_id) = &project.manager_id
        && !manager_id.is_empty()
        && let Err(e) = validation_service.validate_person_exists(manager_id).await
    {
        let response = ApiResponse::<Project> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        )
            .into_response();
    }

    // Check if exists
    match project_repo.exists(&id).await {
        Ok(true) => {
            match project_repo.get(&id).await {
                Ok(Some(existing_project)) => {
                    if !perm_ctx.allows_resource_with_scope(
                        &ResourceType::Project,
                        &PermissionAction::Update,
                        existing_project.created_by.as_deref(),
                        Some(existing_project.id.as_str()),
                        &[],
                    ) || !perm_ctx.allows_action_with_scope(
                        &ResourceType::Project,
                        &PermissionAction::Update,
                        Some(id.as_str()),
                        &[],
                    ) {
                        let response = ApiResponse::<Project> {
                            status: 403,
                            message: "Forbidden: insufficient permission to update this project"
                                .to_string(),
                            data: None,
                        };
                        return (StatusCode::FORBIDDEN, Json(response)).into_response();
                    }

                    // Preserve creation time, ownership and ID
                    project.id = id;
                    project.created_by = existing_project.created_by;
                    project.created_at = existing_project.created_at;
                    project.updated_at = Utc::now().to_rfc3339();

                    match project_repo.save(&project).await {
                        Ok(_) => {
                            let response = ApiResponse {
                                status: 200,
                                message: "Project updated successfully".to_string(),
                                data: Some(project),
                            };
                            (StatusCode::OK, Json(response)).into_response()
                        }
                        Err(e) => {
                            let response = ApiResponse::<Project> {
                                status: 500,
                                message: format!("Failed to update project: {}", e),
                                data: None,
                            };
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                        }
                    }
                }
                _ => {
                    let response = ApiResponse::<Project> {
                        status: 404,
                        message: "Project not found".to_string(),
                        data: None,
                    };
                    (StatusCode::NOT_FOUND, Json(response)).into_response()
                }
            }
        }
        Ok(false) => {
            let response = ApiResponse::<Project> {
                status: 404,
                message: "Project not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Project> {
                status: 500,
                message: format!("Failed to check project existence: {}", e),
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
    async fn test_create_project() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(
            &app.router,
            "/api/v1/projects",
            Some(&app.admin_token),
            json!({
                "name": "Project Alpha",
                "code": "PRJ-001",
                "department": "Engineering"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "body: {:?}", body);
        assert_eq!(body["data"]["name"], "Project Alpha");
    }

    #[tokio::test]
    async fn test_list_projects() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, _) = make_post(
            &app.router,
            "/api/v1/projects",
            Some(&app.admin_token),
            json!({
                "name": "List Project",
                "code": "PRJ-LIST"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);

        let (status, body) =
            make_get(&app.router, "/api/v1/projects", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert!(body["data"]["total"].as_u64().unwrap_or(0) >= 1);
    }

    #[tokio::test]
    async fn test_get_project() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/projects",
            Some(&app.admin_token),
            json!({
                "name": "Get Project",
                "code": "PRJ-GET"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let project_id = create_body["data"]["id"].as_str().unwrap().to_string();

        let (status, body) = make_get(
            &app.router,
            &format!("/api/v1/projects/{}", project_id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert_eq!(body["data"]["name"], "Get Project");
    }

    #[tokio::test]
    async fn test_delete_project() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/projects",
            Some(&app.admin_token),
            json!({
                "name": "Delete Project",
                "code": "PRJ-DEL"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let project_id = create_body["data"]["id"].as_str().unwrap().to_string();

        let (status, _) = make_delete(
            &app.router,
            &format!("/api/v1/projects/{}", project_id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "expected 200 OK on delete");

        let (get_status, _) = make_get(
            &app.router,
            &format!("/api/v1/projects/{}", project_id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(get_status, StatusCode::NOT_FOUND);
    }
}

/// Delete a project
pub async fn delete_project(
    Path(id): Path<String>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    if let Ok(Some(existing)) = project_repo.get(&id).await
        && !perm_ctx.allows_resource_with_scope(
            &ResourceType::Project,
            &PermissionAction::Delete,
            existing.created_by.as_deref(),
            Some(existing.id.as_str()),
            &[],
        )
    {
        let response = ApiResponse::<()> {
            status: 403,
            message: "Forbidden: insufficient permission to delete this project".to_string(),
            data: None,
        };
        return (StatusCode::FORBIDDEN, Json(response)).into_response();
    }

    // Check if any clients are using this project
    match client_repo.count_by_project(&id).await {
        Ok(count) if count > 0 => {
            let response = ApiResponse::<()> {
                status: 400,
                message: format!(
                    "Cannot delete project: {} clients are still assigned to it",
                    count
                ),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to check project usage: {}", e),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
        _ => {}
    }

    match project_repo.delete(&id).await {
        Ok(_) => {
            let response = ApiResponse::<()> {
                status: 200,
                message: "Project deleted successfully".to_string(),
                data: None,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to delete project: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
