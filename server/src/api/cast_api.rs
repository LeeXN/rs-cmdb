use crate::middleware::permission::PermissionContext;
use crate::service::cast_recorder::CastRecorderInner;
use crate::service::execution_session_service::ExecutionSessionService;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum_macros::debug_handler;
use common::entity::execution::ExecutionSession;
use common::models::{ApiResponse, CommandQuery, PaginatedResult};
use std::sync::Arc;
use tracing::instrument;

/// GET /api/v1/sessions
#[debug_handler]
#[instrument(skip(session_svc, perm_ctx))]
pub async fn list_sessions(
    Query(mut params): Query<CommandQuery>,
    Extension(session_svc): Extension<Arc<ExecutionSessionService>>,
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
        session_svc.list_user_sessions(&perm_ctx.user_id, &params).await
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
#[instrument(skip(session_svc, perm_ctx))]
pub async fn get_session(
    Path(session_id): Path<String>,
    Extension(session_svc): Extension<Arc<ExecutionSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match session_svc.get_session(&session_id).await {
        Ok(Some(session)) => {
            if !perm_ctx.is_admin() && session.user_id != perm_ctx.user_id {
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
#[instrument(skip(session_svc, perm_ctx, headers))]
pub async fn get_cast_file(
    Path(session_id): Path<String>,
    Extension(session_svc): Extension<Arc<ExecutionSessionService>>,
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

    if !perm_ctx.is_admin() && session.user_id != perm_ctx.user_id {
        return json_response(403, "Forbidden");
    }

    if !CastRecorderInner::cast_file_exists(&session_id) {
        return json_response(404, "Cast file not found");
    }

    let file_size = match CastRecorderInner::cast_file_size(&session_id) {
        Ok(s) => s,
        Err(_) => return json_response(500, "Failed to read cast file"),
    };

    let range_header = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok());

    let (status_code, body, content_range) = if let Some(r) = range_header {
        const PREFIX: &str = "bytes=";
        if let Some(range_str) = r.strip_prefix(PREFIX) {
            if let Some((start_str, end_str)) = range_str.split_once('-') {
                let start: u64 = start_str.parse().unwrap_or(0);
                let end: u64 = if end_str.is_empty() {
                    (start + 65536).min(file_size.saturating_sub(1))
                } else {
                    end_str.parse().unwrap_or(file_size.saturating_sub(1))
                };
                let length = end - start + 1;
                match CastRecorderInner::read_cast_file_range(&session_id, start, length) {
                    Ok(data) => (
                        StatusCode::PARTIAL_CONTENT,
                        Body::from(data),
                        Some(format!("bytes {}-{}/{}", start, end, file_size)),
                    ),
                    Err(_) => return json_response(500, "Failed to read cast file range"),
                }
            } else {
                (StatusCode::OK, Body::from(CastRecorderInner::read_cast_file(&session_id).unwrap_or_default()), None)
            }
        } else {
            (StatusCode::OK, Body::from(CastRecorderInner::read_cast_file(&session_id).unwrap_or_default()), None)
        }
    } else {
        (StatusCode::OK, Body::from(CastRecorderInner::read_cast_file(&session_id).unwrap_or_default()), None)
    };

    let mut resp = Response::new(body);
    *resp.status_mut() = status_code;
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    if let Some(cr) = content_range {
        resp.headers_mut().insert(
            header::CONTENT_RANGE,
            cr.parse().unwrap(),
        );
    }
    resp.headers_mut().insert(
        header::ACCEPT_RANGES,
        "bytes".parse().unwrap(),
    );
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
