use std::sync::Arc;
use axum::{
    extract::{Path, Extension, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use common::models::{Rack, ApiResponse, RackQuery, PaginatedResult};
use crate::repository::rack_repository::RackRepository;
use crate::repository::client_repository::ClientRepository;
use uuid::Uuid;
use chrono::Utc;

/// List all racks
pub async fn list_racks(
    Query(query): Query<RackQuery>,
    Extension(rack_repo): Extension<Arc<RackRepository>>,
) -> impl IntoResponse {
    match rack_repo.list_all().await {
        Ok(mut racks) => {
            // Filter by search term
            if let Some(ref search) = query.search {
                let search_lower = search.to_lowercase();
                racks.retain(|r| {
                    r.name.to_lowercase().contains(&search_lower) ||
                    r.location.as_ref().map_or(false, |l| l.to_lowercase().contains(&search_lower)) ||
                    r.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&search_lower))
                });
            }

            // Filter by location
            if let Some(ref location) = query.location {
                if !location.is_empty() {
                    racks.retain(|r| r.location.as_ref().map_or(false, |l| l == location));
                }
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
        },
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
        },
        Ok(None) => {
            let response = ApiResponse::<()> {
                status: 404,
                message: "Rack not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        },
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
    Extension(rack_repo): Extension<Arc<RackRepository>>,
    Json(mut rack): Json<Rack>,
) -> impl IntoResponse {
    // Ensure ID is set
    if rack.id.is_empty() {
        rack.id = Uuid::new_v4().to_string();
    }
    
    // Set timestamps
    let now = Utc::now().to_rfc3339();
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
        },
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
    Extension(rack_repo): Extension<Arc<RackRepository>>,
    Json(mut rack): Json<Rack>,
) -> impl IntoResponse {
    // Check if rack exists
    match rack_repo.exists(&id).await {
        Ok(true) => {
            // Ensure ID matches
            rack.id = id;
            // Update timestamp
            rack.updated_at = Utc::now().to_rfc3339();
            
            // We should preserve created_at if possible, but for now we trust the client or just overwrite
            // Ideally we fetch first, but for simplicity we just save
            
            match rack_repo.save(&rack).await {
                Ok(_) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Rack updated successfully".to_string(),
                        data: Some(rack),
                    };
                    (StatusCode::OK, Json(response)).into_response()
                },
                Err(e) => {
                    let response = ApiResponse::<()> {
                        status: 500,
                        message: format!("Failed to update rack: {}", e),
                        data: None,
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                }
            }
        },
        Ok(false) => {
            let response = ApiResponse::<()> {
                status: 404,
                message: "Rack not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        },
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to check rack existence: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Delete a rack
pub async fn delete_rack(
    Path(id): Path<String>,
    Extension(rack_repo): Extension<Arc<RackRepository>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    // Check if any clients are using this rack
    match client_repo.count_by_rack(&id).await {
        Ok(count) if count > 0 => {
             let response = ApiResponse::<()> {
                status: 400,
                message: format!("Cannot delete rack: {} clients are still assigned to it", count),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        },
        Err(e) => {
             let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to check rack usage: {}", e),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        },
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
        },
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
//     }
// }
