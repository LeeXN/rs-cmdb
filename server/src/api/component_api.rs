use crate::repository::client_repository::ClientRepository;
use crate::repository::component_repository::ComponentRepository;
use crate::service::validation_service::ValidationService;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use common::models::{ApiResponse, Component, ComponentStatus, ComponentType, PaginatedResult};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, instrument};

#[derive(Debug, Deserialize)]
pub struct ComponentQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub client_id: Option<String>,
    pub component_type: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchCreateRequest {
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize)]
pub struct BatchUpdateRequest {
    pub ids: Vec<String>,
    pub status: Option<ComponentStatus>,
}

/// List components with filtering
#[debug_handler]
#[instrument(skip(component_repo, client_repo))]
pub async fn list_components(
    Query(params): Query<ComponentQuery>,
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    // Map API query to Repository query
    let status = if let Some(s) = &params.status {
        if s == "all" || s.is_empty() {
            None
        } else {
            match s.as_str() {
                "InStock" => Some(ComponentStatus::InStock),
                "InUse" => Some(ComponentStatus::InUse),
                "Faulty" => Some(ComponentStatus::Faulty),
                "Decommissioned" => Some(ComponentStatus::Decommissioned),
                "LentOut" => Some(ComponentStatus::LentOut),
                _ => Some(ComponentStatus::Unknown),
            }
        }
    } else {
        None
    };

    let component_type = if let Some(t) = &params.component_type {
        if t == "all" || t.is_empty() {
            None
        } else {
            match t.as_str() {
                "GPU" => Some(ComponentType::GPU),
                "CPU" => Some(ComponentType::CPU),
                "Memory" => Some(ComponentType::Memory),
                "Disk" => Some(ComponentType::Disk),
                "NetworkCard" => Some(ComponentType::NetworkCard),
                "Motherboard" => Some(ComponentType::Motherboard),
                "PowerSupply" => Some(ComponentType::PowerSupply),
                _ => Some(ComponentType::Other),
            }
        }
    } else {
        None
    };

    let repo_query = common::models::ComponentQuery {
        page: params.page,
        page_size: params.page_size,
        status,
        component_type,
        search: params.q.clone(),
        client_id: params.client_id.clone(),
    };

    match component_repo.find_with_query(repo_query).await {
        Ok(mut paginated_result) => {
            // Populate client_hostname
            for component in &mut paginated_result.items {
                if let Some(client_id) = &component.client_id
                    && let Ok(Some(client)) = client_repo.get(client_id).await
                {
                    component.client_hostname = Some(client.hostname);
                }
            }

            info!(
                "Listed {} components (page {}/{})",
                paginated_result.items.len(),
                paginated_result.page,
                paginated_result.total_pages
            );
            let response = ApiResponse {
                status: 200,
                message: "Components retrieved successfully".to_string(),
                data: Some(paginated_result),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to list components: {}", err);
            let response = ApiResponse::<PaginatedResult<Component>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Get a component by ID
#[debug_handler]
#[instrument(skip(component_repo))]
pub async fn get_component(
    Path(component_id): Path<String>,
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
) -> impl IntoResponse {
    match component_repo.get(&component_id).await {
        Ok(Some(component)) => {
            let response = ApiResponse {
                status: 200,
                message: "Component retrieved successfully".to_string(),
                data: Some(component),
            };

            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = ApiResponse::<Component> {
                status: 404,
                message: format!("Component {} not found", component_id),
                data: None,
            };

            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to get component {}: {}", component_id, err);
            let response = ApiResponse::<Component> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Create a new component
#[debug_handler]
#[instrument(skip(component_repo, validation_service))]
pub async fn create_component(
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Json(component): Json<Component>,
) -> impl IntoResponse {
    info!("Creating component: {}", component.serial_number);

    // Validate client_id
    if let Some(client_id) = &component.client_id
        && !client_id.is_empty()
        && let Err(e) = validation_service.validate_client_exists(client_id).await
    {
        let response = ApiResponse::<Component> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    match component_repo.save(&component).await {
        Ok(_) => {
            let response = ApiResponse {
                status: 201,
                message: "Component created successfully".to_string(),
                data: Some(component),
            };
            (StatusCode::CREATED, Json(response))
        }
        Err(err) => {
            error!("Failed to create component: {}", err);
            let response = ApiResponse::<Component> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Update a component
#[debug_handler]
#[instrument(skip(component_repo, validation_service))]
pub async fn update_component(
    Path(component_id): Path<String>,
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Json(component): Json<Component>,
) -> impl IntoResponse {
    info!("Updating component: {}", component_id);

    // Validate client_id
    if let Some(client_id) = &component.client_id
        && !client_id.is_empty()
        && let Err(e) = validation_service.validate_client_exists(client_id).await
    {
        let response = ApiResponse::<Component> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    // Check if component exists
    match component_repo.exists(&component_id).await {
        Ok(true) => {
            // Ensure ID matches
            if component.id != component_id {
                let response = ApiResponse::<Component> {
                    status: 400,
                    message: "Component ID mismatch".to_string(),
                    data: None,
                };
                return (StatusCode::BAD_REQUEST, Json(response));
            }

            match component_repo.save(&component).await {
                Ok(_) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Component updated successfully".to_string(),
                        data: Some(component),
                    };
                    (StatusCode::OK, Json(response))
                }
                Err(err) => {
                    error!("Failed to update component {}: {}", component_id, err);
                    let response = ApiResponse::<Component> {
                        status: err.status_code(),
                        message: err.to_string(),
                        data: None,
                    };
                    (
                        StatusCode::from_u16(err.status_code())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(response),
                    )
                }
            }
        }
        Ok(false) => {
            let response = ApiResponse::<Component> {
                status: 404,
                message: format!("Component {} not found", component_id),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to check component {}: {}", component_id, err);
            let response = ApiResponse::<Component> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}
/// Batch create components
#[debug_handler]
#[instrument(skip(component_repo))]
pub async fn batch_create_components(
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Json(request): Json<BatchCreateRequest>,
) -> impl IntoResponse {
    info!("Batch creating {} components", request.components.len());

    let mut created_count = 0;
    let mut errors = Vec::new();

    for component in request.components {
        match component_repo.save(&component).await {
            Ok(_) => created_count += 1,
            Err(e) => errors.push(format!(
                "Failed to save component {}: {}",
                component.serial_number, e
            )),
        }
    }

    if errors.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: format!("Successfully created {} components", created_count),
            data: Some(created_count),
        };
        (StatusCode::OK, Json(response))
    } else {
        let response = ApiResponse {
            status: 207, // Multi-Status
            message: format!(
                "Created {} components, {} failed. Errors: {:?}",
                created_count,
                errors.len(),
                errors
            ),
            data: Some(created_count),
        };
        (StatusCode::MULTI_STATUS, Json(response))
    }
}

/// Batch update components
#[debug_handler]
#[instrument(skip(component_repo))]
pub async fn batch_update_components(
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Json(request): Json<BatchUpdateRequest>,
) -> impl IntoResponse {
    info!("Batch updating {} components", request.ids.len());

    let mut updated_count = 0;
    let mut errors = Vec::new();

    for id in request.ids {
        match component_repo.get(&id).await {
            Ok(Some(mut component)) => {
                if let Some(status) = &request.status {
                    component.status = status.clone();
                }

                match component_repo.save(&component).await {
                    Ok(_) => updated_count += 1,
                    Err(e) => errors.push(format!("Failed to update component {}: {}", id, e)),
                }
            }
            Ok(None) => errors.push(format!("Component {} not found", id)),
            Err(e) => errors.push(format!("Failed to get component {}: {}", id, e)),
        }
    }

    if errors.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: format!("Successfully updated {} components", updated_count),
            data: Some(updated_count),
        };
        (StatusCode::OK, Json(response))
    } else {
        let response = ApiResponse {
            status: 207, // Multi-Status
            message: format!(
                "Updated {} components, {} failed. Errors: {:?}",
                updated_count,
                errors.len(),
                errors
            ),
            data: Some(updated_count),
        };
        (StatusCode::MULTI_STATUS, Json(response))
    }
}

/// Batch delete components
#[debug_handler]
#[instrument(skip(component_repo))]
pub async fn batch_delete_components(
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Json(request): Json<BatchDeleteRequest>,
) -> impl IntoResponse {
    info!("Batch deleting {} components", request.ids.len());

    let mut deleted_count = 0;
    let mut errors = Vec::new();

    for id in request.ids {
        match component_repo.delete(&id).await {
            Ok(_) => deleted_count += 1,
            Err(e) => errors.push(format!("Failed to delete component {}: {}", id, e)),
        }
    }

    if errors.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: format!("Successfully deleted {} components", deleted_count),
            data: Some(deleted_count),
        };
        (StatusCode::OK, Json(response))
    } else {
        let response = ApiResponse {
            status: 207, // Multi-Status
            message: format!(
                "Deleted {} components, {} failed. Errors: {:?}",
                deleted_count,
                errors.len(),
                errors
            ),
            data: Some(deleted_count),
        };
        (StatusCode::MULTI_STATUS, Json(response))
    }
}
