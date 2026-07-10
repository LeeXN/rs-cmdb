use crate::repository::approval_repository::ApprovalRepository;
use crate::service::approval_service::ApprovalService;
use crate::service::command_service::CommandService;
use axum::extract::Path;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use common::entity::permission::{ApprovalPayload, PendingApproval};
use common::entity::user::User;
use common::models::{ApiResponse, ApprovalSummaryResponse, CreateCommandRequest, CreateCommandResponse};
use std::sync::Arc;
use tracing::{error, info, instrument};

#[debug_handler]
#[instrument(skip(svc))]
pub async fn list_pending_approvals(
    Extension(svc): Extension<Arc<ApprovalService>>,
) -> impl IntoResponse {
    match svc.list_all().await {
        Ok(approvals) => {
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Pending approvals retrieved successfully".to_string(),
                data: Some(approvals),
            }))
        }
        Err(e) => {
            error!("Failed to list approvals: {}", e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<Vec<PendingApproval>> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(svc))]
pub async fn list_my_approvals(
    Extension(svc): Extension<Arc<ApprovalService>>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    match svc.list_by_user(&user.id).await {
        Ok(approvals) => {
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Your approvals retrieved successfully".to_string(),
                data: Some(approvals),
            }))
        }
        Err(e) => {
            error!("Failed to list my approvals: {}", e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<Vec<PendingApproval>> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(svc))]
pub async fn get_pending_approval(
    Extension(svc): Extension<Arc<ApprovalService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match svc.get(&id).await {
        Ok(Some(approval)) => {
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Pending approval retrieved successfully".to_string(),
                data: Some(approval),
            }))
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(ApiResponse::<PendingApproval> {
                status: 404,
                message: format!("Pending approval {} not found", id),
                data: None,
            }))
        }
        Err(e) => {
            error!("Failed to get approval {}: {}", id, e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<PendingApproval> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(svc, command_svc))]
pub async fn approve_request(
    Extension(svc): Extension<Arc<ApprovalService>>,
    Extension(command_svc): Extension<Arc<CommandService>>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match svc.approve(&id, &user.username).await {
        Ok(approval) => {
            let mut response = CreateCommandResponse {
                task_id: None,
                approval_id: Some(approval.id.clone()),
                danger_level: "approved".to_string(),
                requires_confirmation: false,
                matched_rule: None,
                message: Some("Approval request approved".to_string()),
            };

            if let Some(ApprovalPayload::Command { request }) = approval.payload.clone() {
                let req = CreateCommandRequest {
                    client_id: request.client_id,
                    command: request.command,
                    shell_mode: request.shell_mode,
                    args: request.args,
                    execution_type: request.execution_type,
                    timeout_secs: request.timeout_secs,
                    force: true,
                };

                match command_svc
                    .create_task_from_approval(
                        &req,
                        &approval.username,
                        &approval.user_id,
                        &request.submitted_role,
                        &request.group_ids,
                    )
                    .await
                {
                    Ok(created) => {
                        if let Some(task_id) = created.task_id.clone() {
                            let _ = svc.attach_executed_task(&approval.id, &task_id).await;
                        }
                        response = created;
                    }
                    Err(e) => {
                        error!("Failed to execute approved request {}: {}", id, e);
                        return (
                            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                            Json(ApiResponse::<CreateCommandResponse> {
                                status: e.status_code(),
                                message: e.log_and_user_message(),
                                data: None,
                            }),
                        );
                    }
                }
            }

            info!("Approval {} approved by {}", id, user.username);
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Approval request approved".to_string(),
                data: Some(response),
            }))
        }
        Err(e) => {
            error!("Failed to approve {}: {}", id, e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<CreateCommandResponse> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(svc))]
pub async fn reject_request(
    Extension(svc): Extension<Arc<ApprovalService>>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match svc.reject(&id, &user.username).await {
        Ok(approval) => {
            info!("Approval {} rejected by {}", id, user.username);
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Approval request rejected".to_string(),
                data: Some(approval),
            }))
        }
        Err(e) => {
            error!("Failed to reject {}: {}", id, e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<PendingApproval> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}

#[debug_handler]
#[instrument(skip(repo))]
pub async fn get_approval_summary(
    Extension(repo): Extension<Arc<ApprovalRepository>>,
) -> impl IntoResponse {
    match repo.list_all().await {
        Ok(approvals) => {
            let mut summary = ApprovalSummaryResponse::default();
            for approval in approvals {
                summary.record_status(&approval.status, approval.executed_task_id.as_ref());
            }
            (StatusCode::OK, Json(ApiResponse {
                status: 200,
                message: "Approval summary retrieved successfully".to_string(),
                data: Some(summary),
            }))
        }
        Err(e) => {
            error!("Failed to build approval summary: {}", e);
            (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
             Json(ApiResponse::<ApprovalSummaryResponse> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }))
        }
    }
}
