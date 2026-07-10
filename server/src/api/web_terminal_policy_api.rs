use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
use axum::extract::Path;
use axum::extract::rejection::JsonRejection;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use common::entity::permission::WebTerminalPolicy;
use common::models::ApiResponse;
use std::sync::Arc;
use tracing::{error, info, instrument};

#[debug_handler]
#[instrument(skip(repo))]
pub async fn list_web_terminal_policies(
    Extension(repo): Extension<Arc<WebTerminalPolicyRepository>>,
) -> impl IntoResponse {
    match repo.list_all().await {
        Ok(policies) => {
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Web terminal policies retrieved successfully".to_string(),
                data: Some(policies),
            }))
        }
        Err(e) => {
            error!("Failed to list web terminal policies: {}", e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<Vec<WebTerminalPolicy>> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn get_web_terminal_policy(
    Extension(repo): Extension<Arc<WebTerminalPolicyRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get(&id).await {
        Ok(Some(policy)) => {
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Web terminal policy retrieved successfully".to_string(),
                data: Some(policy),
            }))
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(ApiResponse::<WebTerminalPolicy> {
                status: 404,
                message: format!("Web terminal policy {} not found", id),
                data: None,
            }))
        }
        Err(e) => {
            error!("Failed to get web terminal policy {}: {}", id, e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<WebTerminalPolicy> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn create_web_terminal_policy(
    Extension(repo): Extension<Arc<WebTerminalPolicyRepository>>,
    result: Result<Json<WebTerminalPolicy>, JsonRejection>,
) -> impl IntoResponse {
    let Json(policy) = match result {
        Ok(p) => p,
        Err(rejection) => {
            error!("Invalid JSON for web terminal policy create: {}", rejection);
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(ApiResponse::<WebTerminalPolicy> {
                status: 422,
                message: format!("Invalid request body: {}", rejection.body_text()),
                data: None,
            }));
        }
    };
    match repo.save(&policy).await {
        Ok(_) => {
            info!("Created web terminal policy: {}", policy.id);
            (StatusCode::CREATED, Json(ApiResponse {
                status: 201,
                message: "Web terminal policy created successfully".to_string(),
                data: Some(policy),
            }))
        }
        Err(e) => {
            error!("Failed to create web terminal policy: {}", e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<WebTerminalPolicy> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn update_web_terminal_policy(
    Extension(repo): Extension<Arc<WebTerminalPolicyRepository>>,
    Path(id): Path<String>,
    Json(policy): Json<WebTerminalPolicy>,
) -> impl IntoResponse {
    if policy.id != id {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::<WebTerminalPolicy> {
            status: 400,
            message: "Web terminal policy ID mismatch".to_string(),
            data: None,
        }));
    }
    match repo.get(&id).await {
        Ok(Some(_)) => match repo.save(&policy).await {
            Ok(_) => {
                info!("Updated web terminal policy: {}", id);
                (StatusCode::OK, Json(ApiResponse {
                    status: 200,
                    message: "Web terminal policy updated successfully".to_string(),
                    data: Some(policy),
                }))
            }
            Err(e) => {
                error!("Failed to update web terminal policy {}: {}", id, e);
                (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(ApiResponse::<WebTerminalPolicy> {
                    status: e.status_code(),
                    message: e.log_and_user_message(),
                    data: None,
                }))
            }
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(ApiResponse::<WebTerminalPolicy> {
                status: 404,
                message: format!("Web terminal policy {} not found", id),
                data: None,
            }))
        }
        Err(e) => {
            error!("Failed to check web terminal policy {}: {}", id, e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<WebTerminalPolicy> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn delete_web_terminal_policy(
    Extension(repo): Extension<Arc<WebTerminalPolicyRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get(&id).await {
        Ok(Some(_)) => match repo.delete(&id).await {
            Ok(_) => {
                info!("Deleted web terminal policy: {}", id);
                (StatusCode::OK, Json(ApiResponse::<()> {
                    status: 200,
                    message: format!("Web terminal policy {} deleted", id),
                    data: None,
                }))
            }
            Err(e) => {
                error!("Failed to delete web terminal policy {}: {}", id, e);
                (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(ApiResponse::<()> {
                    status: e.status_code(),
                    message: e.log_and_user_message(),
                    data: None,
                }))
            }
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(ApiResponse::<()> {
                status: 404,
                message: format!("Web terminal policy {} not found", id),
                data: None,
            }))
        }
        Err(e) => {
            error!("Failed to check web terminal policy {}: {}", id, e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}
