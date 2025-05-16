use std::sync::Arc;
use axum::{
    extract::{Path, Extension, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use common::entity::dictionary::Dictionary;
use common::models::ApiResponse;
use crate::repository::dictionary_repository::DictionaryRepository;
use uuid::Uuid;
use chrono::Utc;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListParams {
    category: Option<String>,
}

/// List dictionary items
pub async fn list_dictionaries(
    Extension(repo): Extension<Arc<DictionaryRepository>>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let result = if let Some(category) = params.category {
        repo.list_by_category(&category).await
    } else {
        repo.list_all().await
    };

    match result {
        Ok(items) => {
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
) -> impl IntoResponse {
    match repo.get(&id).await {
        Ok(Some(item)) => {
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
    Json(mut item): Json<Dictionary>,
) -> impl IntoResponse {
    // Ensure ID is set
    if item.id.is_empty() {
        item.id = Uuid::new_v4().to_string();
    }
    
    // Set timestamps
    let now = Utc::now().to_rfc3339();
    item.created_at = now.clone();
    item.updated_at = now;
    
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
    Json(mut item): Json<Dictionary>,
) -> impl IntoResponse {
    // Check if exists
    match repo.get(&id).await {
        Ok(Some(existing_item)) => {
            // Preserve creation time and ID
            item.id = id;
            item.created_at = existing_item.created_at;
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
) -> impl IntoResponse {
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
