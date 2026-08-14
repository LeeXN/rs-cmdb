use crate::repository::approval_repository::ApprovalRepository;
use crate::repository::exec_policy_repository::ExecPolicyRepository;
use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
use crate::service::cast_recorder::CastRecorderInner;
use crate::service::command_service::CommandService;
use crate::service::terminal_session_service::TerminalSessionService;
use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use axum_macros::debug_handler;
use common::models::{
    ApiResponse, ApprovalSummaryResponse, CastCleanupRequest, CastCleanupResponse,
    RemoteExecOpsOverviewResponse, TerminalOpsSummaryResponse, TerminalSessionSummary,
};
use std::sync::Arc;
use tracing::{error, instrument};

#[debug_handler]
#[instrument(skip(
    command_svc,
    exec_policy_repo,
    web_terminal_policy_repo,
    approval_repo,
    terminal_svc
))]
pub async fn get_overview(
    Extension(command_svc): Extension<Arc<CommandService>>,
    Extension(exec_policy_repo): Extension<Arc<ExecPolicyRepository>>,
    Extension(web_terminal_policy_repo): Extension<Arc<WebTerminalPolicyRepository>>,
    Extension(approval_repo): Extension<Arc<ApprovalRepository>>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
) -> impl IntoResponse {
    let remote_exec_enabled = match command_svc.get_config().await {
        Ok(cfg) => cfg.enabled,
        Err(e) => {
            error!("Failed to load remote exec config: {}", e);
            return error_response::<RemoteExecOpsOverviewResponse>(500, &e.log_and_user_message());
        }
    };
    let exec_policies_count = match exec_policy_repo.count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count exec policies: {}", e);
            return error_response::<RemoteExecOpsOverviewResponse>(
                e.status_code(),
                &e.log_and_user_message(),
            );
        }
    };
    let web_terminal_policies_count = match web_terminal_policy_repo.count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count web terminal policies: {}", e);
            return error_response::<RemoteExecOpsOverviewResponse>(
                e.status_code(),
                &e.log_and_user_message(),
            );
        }
    };
    let approval_summary = match approval_repo.list_all().await {
        Ok(approvals) => summarize_approvals(&approvals),
        Err(e) => {
            error!("Failed to summarize approvals: {}", e);
            return error_response::<RemoteExecOpsOverviewResponse>(
                e.status_code(),
                &e.log_and_user_message(),
            );
        }
    };
    let terminal_summary = match terminal_svc.terminal_ops_summary().await {
        Ok(summary) => summary,
        Err(e) => {
            error!("Failed to summarize terminal sessions: {}", e);
            return error_response::<RemoteExecOpsOverviewResponse>(
                e.status_code(),
                &e.log_and_user_message(),
            );
        }
    };

    success_response(
        StatusCode::OK,
        "Remote exec overview retrieved successfully",
        RemoteExecOpsOverviewResponse {
            remote_exec_enabled,
            exec_policies_count,
            web_terminal_policies_count,
            approval_summary,
            terminal_summary,
            cast_storage_dir: CastRecorderInner::cast_dir().display().to_string(),
        },
    )
}

#[debug_handler]
#[instrument(skip(terminal_svc))]
pub async fn get_terminal_summary(
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
) -> impl IntoResponse {
    match terminal_svc.terminal_ops_summary().await {
        Ok(summary) => success_response(
            StatusCode::OK,
            "Terminal summary retrieved successfully",
            summary,
        ),
        Err(e) => {
            error!("Failed to load terminal summary: {}", e);
            error_response::<TerminalOpsSummaryResponse>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(terminal_svc))]
pub async fn list_terminal_sessions(
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
) -> impl IntoResponse {
    match terminal_svc.list_all_sessions().await {
        Ok(sessions) => success_response(
            StatusCode::OK,
            "Terminal sessions retrieved successfully",
            sessions,
        ),
        Err(e) => {
            error!("Failed to list terminal sessions: {}", e);
            error_response::<Vec<TerminalSessionSummary>>(
                e.status_code(),
                &e.log_and_user_message(),
            )
        }
    }
}

#[debug_handler]
#[instrument]
pub async fn cleanup_casts(Json(req): Json<CastCleanupRequest>) -> impl IntoResponse {
    if req.retention_days == 0 {
        return error_response::<CastCleanupResponse>(400, "retention_days must be greater than 0");
    }

    match CastRecorderInner::delete_old_casts(req.retention_days) {
        Ok(deleted_files) => success_response(
            StatusCode::OK,
            "Cast history cleanup completed",
            CastCleanupResponse {
                retention_days: req.retention_days,
                deleted_files,
                cast_storage_dir: CastRecorderInner::cast_dir().display().to_string(),
            },
        ),
        Err(e) => {
            error!("Failed to clean cast history: {}", e);
            error_response::<CastCleanupResponse>(e.status_code(), &e.log_and_user_message())
        }
    }
}

fn summarize_approvals(
    approvals: &[common::entity::permission::PendingApproval],
) -> ApprovalSummaryResponse {
    let mut summary = ApprovalSummaryResponse::default();
    for approval in approvals {
        summary.record_status(&approval.status, approval.executed_task_id.as_ref());
    }
    summary
}

fn success_response<T: PartialEq>(
    status: StatusCode,
    message: &str,
    data: T,
) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        status,
        Json(ApiResponse {
            status: status.as_u16(),
            message: message.to_string(),
            data: Some(data),
        }),
    )
}

fn error_response<T: PartialEq>(status: u16, message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        Json(ApiResponse {
            status,
            message: message.to_string(),
            data: None,
        }),
    )
}
