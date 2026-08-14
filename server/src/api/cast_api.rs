use crate::middleware::permission::PermissionContext;
use crate::repository::client_repository::ClientRepository;
use crate::service::cast_recorder::CastRecorderInner;
use crate::service::execution_session_service::ExecutionSessionService;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum_macros::debug_handler;
use common::entity::execution::ExecutionSession;
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::models::{ApiResponse, Client, CommandQuery, PaginatedResult};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::instrument;

/// GET /api/v1/sessions
#[debug_handler]
#[instrument(skip(session_svc, client_repo, perm_ctx))]
pub async fn list_sessions(
    Query(mut params): Query<CommandQuery>,
    Extension(session_svc): Extension<Arc<ExecutionSessionService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    params.page = Some(params.page.unwrap_or(1).max(1));
    params.page_size = Some(params.page_size.unwrap_or(20).clamp(1, 100));

    let result = if perm_ctx.is_admin() {
        if let Some(uid) = params.user_id.clone().filter(|uid| !uid.is_empty()) {
            session_svc.list_user_sessions(&uid, &params).await
        } else {
            session_svc.list_sessions(&params).await
        }
    } else {
        params.user_id = Some(perm_ctx.user_id.clone());
        match allowed_client_ids_for_session_scope(&client_repo, &perm_ctx).await {
            Ok(allowed_client_ids) => {
                session_svc
                    .list_user_sessions_for_client_ids(
                        &perm_ctx.user_id,
                        &params,
                        &allowed_client_ids,
                    )
                    .await
            }
            Err(message) => {
                return json_paginated_error(403, &message);
            }
        }
    };

    match result {
        Ok(sessions) => {
            let response = ApiResponse {
                status: 200,
                message: "Sessions retrieved".to_string(),
                data: Some(sessions),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<PaginatedResult<ExecutionSession>> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/sessions/{id}
#[debug_handler]
#[instrument(skip(session_svc, client_repo, perm_ctx))]
pub async fn get_session(
    Path(session_id): Path<String>,
    Extension(session_svc): Extension<Arc<ExecutionSessionService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match session_svc.get_session(&session_id).await {
        Ok(Some(session)) => {
            if !can_view_session(&session, &client_repo, &perm_ctx).await {
                let response = ApiResponse::<ExecutionSession> {
                    status: 403,
                    message: "Forbidden: you can only view your own sessions".to_string(),
                    data: None,
                };
                return (StatusCode::FORBIDDEN, Json(response)).into_response();
            }
            let response = ApiResponse {
                status: 200,
                message: "Session retrieved".to_string(),
                data: Some(session),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::<ExecutionSession> {
                status: 404,
                message: "Session not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<ExecutionSession> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/sessions/{id}/cast
#[debug_handler]
#[instrument(skip(session_svc, client_repo, perm_ctx, headers))]
pub async fn get_cast_file(
    Path(session_id): Path<String>,
    Extension(session_svc): Extension<Arc<ExecutionSessionService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    headers: axum::http::HeaderMap,
) -> Response {
    let session = match session_svc.get_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return json_response(404, "Session not found");
        }
        Err(e) => {
            return json_response(e.status_code(), &e.log_and_user_message());
        }
    };

    if !can_view_session(&session, &client_repo, &perm_ctx).await {
        return json_response(403, "Forbidden");
    }

    if !CastRecorderInner::cast_file_exists(&session_id) {
        return json_response(404, "Cast file not found");
    }

    let file_size = match CastRecorderInner::cast_file_size(&session_id) {
        Ok(s) => s,
        Err(_) => return json_response(500, "Failed to read cast file"),
    };

    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    let (status_code, body, content_range) = if let Some(r) = range_header {
        const PREFIX: &str = "bytes=";
        const MAX_RANGE_LENGTH: u64 = 1024 * 1024;
        let Some(range_str) = r.strip_prefix(PREFIX) else {
            return json_response(416, "Invalid byte range");
        };
        let Some((start_str, end_str)) = range_str.split_once('-') else {
            return json_response(416, "Invalid byte range");
        };
        if file_size == 0 || range_str.contains(',') {
            return json_response(416, "Invalid byte range");
        }

        let (start, requested_end) = if start_str.is_empty() {
            // Suffix range: bytes=-N
            let Ok(suffix_length) = end_str.parse::<u64>() else {
                return json_response(416, "Invalid byte range");
            };
            if suffix_length == 0 {
                return json_response(416, "Invalid byte range");
            }
            let length = suffix_length.min(MAX_RANGE_LENGTH).min(file_size);
            (file_size - length, file_size - 1)
        } else {
            let Ok(start) = start_str.parse::<u64>() else {
                return json_response(416, "Invalid byte range");
            };
            if start >= file_size {
                return json_response(416, "Byte range starts beyond the cast file");
            }
            let requested_end = if end_str.is_empty() {
                start.saturating_add(MAX_RANGE_LENGTH).saturating_sub(1)
            } else {
                let Ok(end) = end_str.parse::<u64>() else {
                    return json_response(416, "Invalid byte range");
                };
                end
            };
            (start, requested_end)
        };

        if requested_end < start {
            return json_response(416, "Invalid byte range");
        }
        let end = requested_end.min(file_size - 1);
        let length = end
            .saturating_sub(start)
            .saturating_add(1)
            .min(MAX_RANGE_LENGTH);
        let end = start + length - 1;
        match CastRecorderInner::read_cast_file_range(&session_id, start, length) {
            Ok(data) => (
                StatusCode::PARTIAL_CONTENT,
                Body::from(data),
                Some(format!("bytes {}-{}/{}", start, end, file_size)),
            ),
            Err(_) => return json_response(500, "Failed to read cast file range"),
        }
    } else {
        (
            StatusCode::OK,
            Body::from(CastRecorderInner::read_cast_file(&session_id).unwrap_or_default()),
            None,
        )
    };

    let mut resp = Response::new(body);
    *resp.status_mut() = status_code;
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    if let Some(cr) = content_range {
        resp.headers_mut()
            .insert(header::CONTENT_RANGE, cr.parse().unwrap());
    }
    resp.headers_mut()
        .insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    resp
}

fn json_response(status: u16, message: &str) -> Response {
    let response = ApiResponse::<()> {
        status,
        message: message.to_string(),
        data: None,
    };
    (
        StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        Json(response),
    )
        .into_response()
}

async fn allowed_client_ids_for_session_scope(
    client_repo: &ClientRepository,
    perm_ctx: &PermissionContext,
) -> Result<HashSet<String>, String> {
    let scope = perm_ctx
        .evaluate(&ResourceType::Client, &PermissionAction::View)
        .map_err(|_| "Unable to resolve client permission scope".to_string())?;
    if matches!(scope, ScopeConstraint::None) {
        return Err("you do not have permission to view execution history".to_string());
    }
    let clients = client_repo
        .list_all()
        .await
        .map_err(|_| "Unable to resolve client permission scope".to_string())?;
    Ok(clients
        .into_iter()
        .filter(|client| client_matches_scope(&scope, perm_ctx, client))
        .map(|client| client.id)
        .collect())
}

async fn can_view_session(
    session: &ExecutionSession,
    client_repo: &ClientRepository,
    perm_ctx: &PermissionContext,
) -> bool {
    if perm_ctx.is_admin() {
        return true;
    }
    if session.user_id != perm_ctx.user_id {
        return false;
    }
    let Ok(scope) = perm_ctx.evaluate(&ResourceType::Client, &PermissionAction::View) else {
        return false;
    };
    if matches!(scope, ScopeConstraint::None) {
        return false;
    }
    if matches!(scope, ScopeConstraint::All) {
        return true;
    }
    let Ok(clients) = client_repo.list_all().await else {
        return false;
    };
    let clients_by_id = clients
        .into_iter()
        .map(|client| (client.id.clone(), client))
        .collect::<std::collections::HashMap<_, _>>();
    session.client_ids.iter().all(|client_id| {
        clients_by_id
            .get(client_id)
            .is_some_and(|client| client_matches_scope(&scope, perm_ctx, client))
    })
}

fn client_matches_scope(
    scope: &ScopeConstraint,
    perm_ctx: &PermissionContext,
    client: &Client,
) -> bool {
    PermissionContext::matches_scope(
        scope,
        &perm_ctx.user_id,
        client.created_by.as_deref(),
        client.project_id.as_deref(),
        &PermissionContext::client_tags(client),
    )
}

fn json_paginated_error(status: u16, message: &str) -> axum::response::Response {
    (
        StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        Json(ApiResponse::<PaginatedResult<ExecutionSession>> {
            status,
            message: message.to_string(),
            data: None,
        }),
    )
        .into_response()
}
