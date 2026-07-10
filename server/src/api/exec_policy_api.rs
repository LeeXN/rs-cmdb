use crate::repository::exec_policy_repository::ExecPolicyRepository;
use axum::extract::Path;
use axum::extract::rejection::JsonRejection;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use common::entity::permission::ExecPolicy;
use common::models::ApiResponse;
use std::sync::Arc;
use tracing::{error, info, instrument};

#[debug_handler]
#[instrument(skip(repo))]
pub async fn list_exec_policies(
    Extension(repo): Extension<Arc<ExecPolicyRepository>>,
) -> impl IntoResponse {
    match repo.list_all().await {
        Ok(policies) => {
            let response = ApiResponse {
                status: 200,
                message: "Exec policies retrieved successfully".to_string(),
                data: Some(policies),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Failed to list exec policies: {}", e);
            let response = ApiResponse::<Vec<ExecPolicy>> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn get_exec_policy(
    Extension(repo): Extension<Arc<ExecPolicyRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get(&id).await {
        Ok(Some(policy)) => {
            let response = ApiResponse {
                status: 200,
                message: "Exec policy retrieved successfully".to_string(),
                data: Some(policy),
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = ApiResponse::<ExecPolicy> {
                status: 404,
                message: format!("Exec policy {} not found", id),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(e) => {
            error!("Failed to get exec policy {}: {}", id, e);
            let response = ApiResponse::<ExecPolicy> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn create_exec_policy(
    Extension(repo): Extension<Arc<ExecPolicyRepository>>,
    result: Result<Json<ExecPolicy>, JsonRejection>,
) -> impl IntoResponse {
    let Json(policy) = match result {
        Ok(p) => p,
        Err(rejection) => {
            error!("Invalid JSON for exec policy create: {}", rejection);
            let response = ApiResponse::<ExecPolicy> {
                status: 422,
                message: format!("Invalid request body: {}", rejection.body_text()),
                data: None,
            };
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(response));
        }
    };
    match repo.save(&policy).await {
        Ok(_) => {
            info!("Created exec policy: {}", policy.id);
            let response = ApiResponse {
                status: 201,
                message: "Exec policy created successfully".to_string(),
                data: Some(policy),
            };
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            error!("Failed to create exec policy: {}", e);
            let response = ApiResponse::<ExecPolicy> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn update_exec_policy(
    Extension(repo): Extension<Arc<ExecPolicyRepository>>,
    Path(id): Path<String>,
    Json(policy): Json<ExecPolicy>,
) -> impl IntoResponse {
    if policy.id != id {
        let response = ApiResponse::<ExecPolicy> {
            status: 400,
            message: "Exec policy ID mismatch".to_string(),
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    match repo.get(&id).await {
        Ok(Some(_)) => {
            match repo.save(&policy).await {
                Ok(_) => {
                    info!("Updated exec policy: {}", id);
                    let response = ApiResponse {
                        status: 200,
                        message: "Exec policy updated successfully".to_string(),
                        data: Some(policy),
                    };
                    (StatusCode::OK, Json(response))
                }
                Err(e) => {
                    error!("Failed to update exec policy {}: {}", id, e);
                    let response = ApiResponse::<ExecPolicy> {
                        status: e.status_code(),
                        message: e.log_and_user_message(),
                        data: None,
                    };
                    (
                        StatusCode::from_u16(e.status_code())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(response),
                    )
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<ExecPolicy> {
                status: 404,
                message: format!("Exec policy {} not found", id),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(e) => {
            error!("Failed to check exec policy {}: {}", id, e);
            let response = ApiResponse::<ExecPolicy> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn delete_exec_policy(
    Extension(repo): Extension<Arc<ExecPolicyRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get(&id).await {
        Ok(Some(_)) => {
            match repo.delete(&id).await {
                Ok(_) => {
                    info!("Deleted exec policy: {}", id);
                    let response = ApiResponse::<()> {
                        status: 200,
                        message: format!("Exec policy {} deleted", id),
                        data: None,
                    };
                    (StatusCode::OK, Json(response))
                }
                Err(e) => {
                    error!("Failed to delete exec policy {}: {}", id, e);
                    let response = ApiResponse::<()> {
                        status: e.status_code(),
                        message: e.log_and_user_message(),
                        data: None,
                    };
                    (
                        StatusCode::from_u16(e.status_code())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(response),
                    )
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()> {
                status: 404,
                message: format!("Exec policy {} not found", id),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(e) => {
            error!("Failed to check exec policy {}: {}", id, e);
            let response = ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}
