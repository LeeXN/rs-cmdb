use axum::{
    extract::{Path, Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use common::models::ApiResponse;
use common::entity::user::{UpdateUserRequest, UserResponse};
use crate::repository::user_repository::UserRepository;
use crate::service::auth_service::AuthService;
use tracing::{info, error};

/// List all users
pub async fn list_users(
    Extension(user_repo): Extension<Arc<UserRepository>>,
) -> impl IntoResponse {
    match user_repo.list_all().await {
        Ok(users) => {
            let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
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
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "User updated successfully".to_string(),
                data: Some(user.into()),
            }),
        ),
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
) -> impl IntoResponse {
    info!("Deleting user: {}", id);
    match user_repo.delete(&id).await {
        Ok(_) => {
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
