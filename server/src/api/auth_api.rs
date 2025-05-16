use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use common::models::ApiResponse;
use common::entity::user::{User, Role, LoginRequest, LoginResponse, CreateUserRequest, ChangePasswordRequest, UserResponse};
use crate::repository::user_repository::UserRepository;
use crate::service::auth_service::AuthService;
use tracing::error;

pub async fn change_password(
    Extension(user_repo): Extension<Arc<UserRepository>>,
    Extension(auth_service): Extension<Arc<AuthService>>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Verify old password
    match auth_service.verify_password(&payload.old_password, &current_user.password_hash) {
        Ok(true) => {},
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
            match auth_service.generate_token(&user) {
                Ok(token) => {
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
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
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

    let user = User {
        id: Uuid::new_v4().to_string(),
        username: payload.username,
        password_hash,
        role: payload.role.unwrap_or(Role::Viewer),
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    match user_repo.save(&user).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(ApiResponse {
                status: 201,
                message: "User registered successfully".to_string(),
                data: Some(user.into()),
            }),
        ),
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

pub async fn me(
    Extension(current_user): Extension<User>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse {
            status: 200,
            message: "Current user info".to_string(),
            data: Some(UserResponse::from(current_user)),
        }),
    )
}
