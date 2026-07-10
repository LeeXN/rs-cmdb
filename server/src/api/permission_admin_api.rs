use crate::repository::exec_policy_repository::ExecPolicyRepository;
use crate::repository::permission_repository::PermissionRepository;
use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
use crate::service::permission_service::PermissionService;
use axum::extract::Path;
use axum::extract::rejection::JsonRejection;
use axum::{
    Json,
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use common::entity::permission::{Group, PermissionRule};
use common::models::{ApiResponse, PermissionOverviewResponse};
use std::sync::Arc;
use tracing::{error, info, instrument};

#[debug_handler]
#[instrument(skip(perm_repo, exec_policy_repo, web_terminal_policy_repo))]
pub async fn get_permission_overview(
    Extension(perm_repo): Extension<Arc<PermissionRepository>>,
    Extension(exec_policy_repo): Extension<Arc<ExecPolicyRepository>>,
    Extension(web_terminal_policy_repo): Extension<Arc<WebTerminalPolicyRepository>>,
) -> impl IntoResponse {
    let rules_count = match perm_repo.rule_count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count permission rules for overview: {}", e);
            return error_response::<PermissionOverviewResponse>(e.status_code(), &e.log_and_user_message());
        }
    };
    let groups_count = match perm_repo.group_count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count groups for overview: {}", e);
            return error_response::<PermissionOverviewResponse>(e.status_code(), &e.log_and_user_message());
        }
    };
    let exec_policies_count = match exec_policy_repo.count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count exec policies: {}", e);
            return error_response::<PermissionOverviewResponse>(e.status_code(), &e.log_and_user_message());
        }
    };
    let web_terminal_policies_count = match web_terminal_policy_repo.count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to count web terminal policies: {}", e);
            return error_response::<PermissionOverviewResponse>(e.status_code(), &e.log_and_user_message());
        }
    };

    success_response(
        StatusCode::OK,
        "Permission overview retrieved successfully",
        PermissionOverviewResponse {
            rules_count,
            groups_count,
            exec_policies_count,
            web_terminal_policies_count,
        },
    )
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn list_rules(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
) -> impl IntoResponse {
    match perm_svc.list_rules().await {
        Ok(rules) => success_response(StatusCode::OK, "Permission rules retrieved successfully", rules),
        Err(e) => {
            error!("Failed to list permission rules: {}", e);
            error_response::<Vec<PermissionRule>>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn get_rule(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match perm_svc.get_rule(&id).await {
        Ok(Some(rule)) => success_response(StatusCode::OK, "Permission rule retrieved successfully", rule),
        Ok(None) => error_response::<PermissionRule>(404, &format!("Permission rule {} not found", id)),
        Err(e) => {
            error!("Failed to get permission rule {}: {}", id, e);
            error_response::<PermissionRule>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn create_rule(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    result: Result<Json<PermissionRule>, JsonRejection>,
) -> impl IntoResponse {
    let Json(rule) = match result {
        Ok(rule) => rule,
        Err(rejection) => {
            error!("Invalid JSON for permission rule create: {}", rejection);
            return error_response::<PermissionRule>(422, &format!("Invalid request body: {}", rejection.body_text()));
        }
    };

    match perm_svc.create_rule(&rule).await {
        Ok(()) => {
            info!("Created permission rule: {}", rule.id);
            success_response(StatusCode::CREATED, "Permission rule created successfully", rule)
        }
        Err(e) => {
            error!("Failed to create permission rule: {}", e);
            error_response::<PermissionRule>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn update_rule(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    Path(id): Path<String>,
    Json(rule): Json<PermissionRule>,
) -> impl IntoResponse {
    if rule.id != id {
        return error_response::<PermissionRule>(400, "Permission rule ID mismatch");
    }

    match perm_svc.get_rule(&id).await {
        Ok(Some(_)) => match perm_svc.update_rule(&rule).await {
            Ok(()) => {
                info!("Updated permission rule: {}", id);
                success_response(StatusCode::OK, "Permission rule updated successfully", rule)
            }
            Err(e) => {
                error!("Failed to update permission rule {}: {}", id, e);
                error_response::<PermissionRule>(e.status_code(), &e.log_and_user_message())
            }
        },
        Ok(None) => error_response::<PermissionRule>(404, &format!("Permission rule {} not found", id)),
        Err(e) => {
            error!("Failed to check permission rule {}: {}", id, e);
            error_response::<PermissionRule>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn delete_rule(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match perm_svc.get_rule(&id).await {
        Ok(Some(_)) => match perm_svc.delete_rule(&id).await {
            Ok(()) => {
                info!("Deleted permission rule: {}", id);
                (StatusCode::OK, Json(ApiResponse::<()> {
                    status: 200,
                    message: format!("Permission rule {} deleted", id),
                    data: None,
                }))
            }
            Err(e) => {
                error!("Failed to delete permission rule {}: {}", id, e);
                error_response::<()>(e.status_code(), &e.log_and_user_message())
            }
        },
        Ok(None) => error_response::<()>(404, &format!("Permission rule {} not found", id)),
        Err(e) => {
            error!("Failed to check permission rule {}: {}", id, e);
            error_response::<()>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn list_groups(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
) -> impl IntoResponse {
    match perm_svc.list_groups().await {
        Ok(groups) => success_response(StatusCode::OK, "Permission groups retrieved successfully", groups),
        Err(e) => {
            error!("Failed to list permission groups: {}", e);
            error_response::<Vec<Group>>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn get_group(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match perm_svc.get_group(&id).await {
        Ok(Some(group)) => success_response(StatusCode::OK, "Permission group retrieved successfully", group),
        Ok(None) => error_response::<Group>(404, &format!("Permission group {} not found", id)),
        Err(e) => {
            error!("Failed to get permission group {}: {}", id, e);
            error_response::<Group>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn create_group(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    result: Result<Json<Group>, JsonRejection>,
) -> impl IntoResponse {
    let Json(group) = match result {
        Ok(group) => group,
        Err(rejection) => {
            error!("Invalid JSON for permission group create: {}", rejection);
            return error_response::<Group>(422, &format!("Invalid request body: {}", rejection.body_text()));
        }
    };

    match perm_svc.create_group(&group).await {
        Ok(()) => {
            info!("Created permission group: {}", group.id);
            success_response(StatusCode::CREATED, "Permission group created successfully", group)
        }
        Err(e) => {
            error!("Failed to create permission group: {}", e);
            error_response::<Group>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn update_group(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    Path(id): Path<String>,
    Json(group): Json<Group>,
) -> impl IntoResponse {
    if group.id != id {
        return error_response::<Group>(400, "Permission group ID mismatch");
    }

    match perm_svc.get_group(&id).await {
        Ok(Some(_)) => match perm_svc.update_group(&group).await {
            Ok(()) => {
                info!("Updated permission group: {}", id);
                success_response(StatusCode::OK, "Permission group updated successfully", group)
            }
            Err(e) => {
                error!("Failed to update permission group {}: {}", id, e);
                error_response::<Group>(e.status_code(), &e.log_and_user_message())
            }
        },
        Ok(None) => error_response::<Group>(404, &format!("Permission group {} not found", id)),
        Err(e) => {
            error!("Failed to check permission group {}: {}", id, e);
            error_response::<Group>(e.status_code(), &e.log_and_user_message())
        }
    }
}

#[debug_handler]
#[instrument(skip(perm_svc))]
pub async fn delete_group(
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match perm_svc.get_group(&id).await {
        Ok(Some(_)) => match perm_svc.delete_group(&id).await {
            Ok(()) => {
                info!("Deleted permission group: {}", id);
                (StatusCode::OK, Json(ApiResponse::<()> {
                    status: 200,
                    message: format!("Permission group {} deleted", id),
                    data: None,
                }))
            }
            Err(e) => {
                error!("Failed to delete permission group {}: {}", id, e);
                error_response::<()>(e.status_code(), &e.log_and_user_message())
            }
        },
        Ok(None) => error_response::<()>(404, &format!("Permission group {} not found", id)),
        Err(e) => {
            error!("Failed to check permission group {}: {}", id, e);
            error_response::<()>(e.status_code(), &e.log_and_user_message())
        }
    }
}

fn success_response<T: PartialEq>(status: StatusCode, message: &str, data: T) -> (StatusCode, Json<ApiResponse<T>>) {
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
