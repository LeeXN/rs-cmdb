//! API handlers for the client agent side of remote command execution.
//!
//! These endpoints are called by the client agent, not the browser.
//! They are placed under a separate path so they can be kept in the
//! *public* routes group (agents use their own auth token, not JWT).
//!
//! Endpoints:
//!   GET  /api/v1/agent/commands/pending               – long-poll for pending task
//!   POST /api/v1/agent/commands/{id}/start            – mark task as running
//!   POST /api/v1/agent/commands/{id}/logs             – push log lines
//!   POST /api/v1/agent/commands/{id}/complete         – mark task completed/failed

use crate::middleware::agent_auth::AuthenticatedAgent;
use crate::service::command_service::CommandService;
use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use common::models::{AgentCompleteRequest, AgentLogRequest, ApiResponse};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

/// How long to hold an empty long-poll before returning 204.
const LONG_POLL_HOLD: Duration = Duration::from_secs(30);
/// How often to check for a new task during long-poll.
const LONG_POLL_INTERVAL: Duration = Duration::from_secs(2);

// ── GET /api/v1/agent/commands/pending?client_id={id} ──────────────────────
//
// The agent calls this repeatedly. The server holds the connection up to 30 s
// and returns 200+task as soon as one is available, or 204 if none arrive.

pub async fn poll_pending(
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(requested_id) = params.get("client_id") {
        if requested_id != &agent.client_id {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()> {
                    status: 403,
                    message: "client_id does not match authenticated agent".into(),
                    data: None,
                }),
            )
                .into_response();
        }
    }
    let client_id = agent.client_id.clone();

    // Long-poll loop
    let start = tokio::time::Instant::now();
    loop {
        match cmd_svc.claim_pending_for_client(&client_id).await {
            Ok(Some(task)) => {
                info!("Dispatching task {} to client {}", task.id, client_id);
                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        status: 200,
                        message: "OK".into(),
                        data: Some(task),
                    }),
                )
                    .into_response();
            }
            Ok(None) => {}
            Err(e) => {
                error!("poll_pending error for {}: {}", client_id, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()> {
                        status: 500,
                        message: e.log_and_user_message(),
                        data: None,
                    }),
                )
                    .into_response();
            }
        }

        if start.elapsed() + LONG_POLL_INTERVAL >= LONG_POLL_HOLD {
            // Timeout – return 204 No Content
            return StatusCode::NO_CONTENT.into_response();
        }
        sleep(LONG_POLL_INTERVAL).await;
    }
}

// ── POST /api/v1/agent/commands/{id}/start ─────────────────────────────────

pub async fn start_command(
    Path(id): Path<String>,
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
) -> impl IntoResponse {
    match cmd_svc.mark_running_for_client(&id, &agent.client_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                status: 200,
                message: "OK".into(),
                data: None,
            }),
        )
            .into_response(),
        Err(e) => {
            let code = e.status_code();
            (
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<()> {
                    status: code,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}

// ── POST /api/v1/agent/commands/{id}/logs ──────────────────────────────────

pub async fn push_logs(
    Path(id): Path<String>,
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
    Json(req): Json<AgentLogRequest>,
) -> impl IntoResponse {
    match cmd_svc
        .push_logs_for_client(&id, &agent.client_id, req)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                status: 200,
                message: "OK".into(),
                data: None,
            }),
        )
            .into_response(),
        Err(e) => {
            error!("push_logs {}: {}", id, e);
            let code = e.status_code();
            (
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<()> {
                    status: code,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}

// ── POST /api/v1/agent/commands/{id}/complete ──────────────────────────────

pub async fn complete_command(
    Path(id): Path<String>,
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
    Json(req): Json<AgentCompleteRequest>,
) -> impl IntoResponse {
    match cmd_svc
        .complete_task_for_client(&id, &agent.client_id, req)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                status: 200,
                message: "OK".into(),
                data: None,
            }),
        )
            .into_response(),
        Err(e) => {
            error!("complete_command {}: {}", id, e);
            let code = e.status_code();
            (
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<()> {
                    status: code,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode},
    };
    use tower::ServiceExt;

    use crate::tests::fixtures::TestAppBuilder;

    async fn make_get(app: &axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    async fn make_post(app: &axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({})).unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    #[tokio::test]
    async fn test_poll_pending_unauthorized() {
        let app = TestAppBuilder::new().build().await;
        let (status, _body) = make_get(&app.router, "/api/v1/agent/commands/pending").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_agent_start_unauthorized() {
        let app = TestAppBuilder::new().build().await;
        let (status, _body) =
            make_post(&app.router, "/api/v1/agent/commands/nonexistent/start").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
