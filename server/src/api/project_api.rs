use crate::repository::client_repository::ClientRepository;
use crate::repository::project_repository::ProjectRepository;
use crate::service::validation_service::ValidationService;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use common::models::{ApiResponse, PaginatedResult, Project, ProjectQuery};
use std::sync::Arc;
use uuid::Uuid;

/// List all projects
pub async fn list_projects(
    Query(query): Query<ProjectQuery>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
) -> impl IntoResponse {
    match project_repo.list_all().await {
        Ok(mut projects) => {
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
) -> impl IntoResponse {
    match project_repo.get(&id).await {
        Ok(Some(project)) => {
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
            message: e.to_string(),
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

    // Set timestamps
    let now = Utc::now().to_rfc3339();
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
            message: e.to_string(),
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
                    // Preserve creation time and ID
                    project.id = id;
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

/// Delete a project
pub async fn delete_project(
    Path(id): Path<String>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
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
