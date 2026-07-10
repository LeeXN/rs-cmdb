//! REST API handlers for remote command execution (browser/operator side).
//!
//! Endpoints:
//!   GET    /api/v1/remote-exec/config                   – get global switch
//!   PUT    /api/v1/remote-exec/config                   – update global switch  (Admin)
//!   POST   /api/v1/remote-exec/commands                 – create task (Admin only)
//!   GET    /api/v1/remote-exec/commands                 – list tasks             (all auth)
//!   GET    /api/v1/remote-exec/commands/{id}            – get task               (all auth)
//!   GET    /api/v1/remote-exec/commands/{id}/logs       – get full log           (all auth)
//!   GET    /api/v1/remote-exec/commands/{id}/stream     – SSE live stream        (all auth)

use crate::service::command_service::CommandService;
use crate::service::sse_hub::SseHub;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use axum::Json;
use common::command::CommandStatus;
use common::entity::user::User;
use common::models::{ApiResponse, CommandTaskListResponse, CreateCommandRequest, UpdateRemoteExecConfigRequest};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub client_id: Option<String>,
}

// ── GET /api/v1/remote-exec/config ─────────────────────────────────────────

pub async fn get_config(
    Extension(cmd_svc): Extension<Arc<CommandService>>,
) -> impl IntoResponse {
    match cmd_svc.get_config().await {
        Ok(cfg) => (
            StatusCode::OK,
            Json(ApiResponse { status: 200, message: "OK".into(), data: Some(cfg) }),
        )
            .into_response(),
        Err(e) => {
            error!("get_config: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: 500,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}

// ── PUT /api/v1/remote-exec/config  (Admin only) ───────────────────────────

pub async fn update_config(
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Extension(user): Extension<User>,
    Json(req): Json<UpdateRemoteExecConfigRequest>,
) -> impl IntoResponse {
    match cmd_svc.update_config(req, &user.username).await {
        Ok(cfg) => (
            StatusCode::OK,
            Json(ApiResponse { status: 200, message: "OK".into(), data: Some(cfg) }),
        )
            .into_response(),
        Err(e) => {
            error!("update_config: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: 500,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}

// ── POST /api/v1/remote-exec/commands  (User+Admin) ────────────────────────

pub async fn create_command(
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Extension(user): Extension<User>,
    Json(req): Json<CreateCommandRequest>,
) -> impl IntoResponse {
    // Admin-only: require_admin middleware ensures only admins reach this handler.
    // allow_force = true so admins can override warnings, but blocked commands are never allowed.
    match cmd_svc.create_task(&req, &user.username, true, &user.id, &user.role, &[]).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(ApiResponse { status: 200, message: "OK".into(), data: Some(resp) }),
        )
            .into_response(),
        Err(e) => {
            let code = e.status_code();
            error!("create_command: {}", e);
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

// ── GET /api/v1/remote-exec/commands  (all auth) ───────────────────────────

pub async fn list_commands(
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20).min(100);
    let client_id = q.client_id.as_deref();
    match cmd_svc.list_tasks(client_id, page, page_size).await {
        Ok((tasks, total)) => {
            let body = CommandTaskListResponse {
                tasks,
                total,
                page,
                page_size,
            };
            (
                StatusCode::OK,
                Json(ApiResponse { status: 200, message: "OK".into(), data: Some(body) }),
            )
                .into_response()
        }
        Err(e) => {
            error!("list_commands: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: 500,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}

// ── GET /api/v1/remote-exec/commands/{id}  (all auth) ──────────────────────

pub async fn get_command(
    Path(id): Path<String>,
    Extension(cmd_svc): Extension<Arc<CommandService>>,
) -> impl IntoResponse {
    match cmd_svc.get_task(&id).await {
        Ok(task) => (
            StatusCode::OK,
            Json(ApiResponse { status: 200, message: "OK".into(), data: Some(task) }),
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

// ── GET /api/v1/remote-exec/commands/{id}/logs  (all auth) ─────────────────

pub async fn get_command_logs(
    Path(id): Path<String>,
    Extension(cmd_svc): Extension<Arc<CommandService>>,
) -> impl IntoResponse {
    match cmd_svc.get_task_logs(&id).await {
        Ok(logs) => (
            StatusCode::OK,
            Json(ApiResponse { status: 200, message: "OK".into(), data: Some(logs) }),
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

// ── GET /api/v1/remote-exec/commands/{id}/stream  (SSE, all auth) ──────────
//
// The client subscribes to this endpoint and receives SSE events:
//   - event: "log"   data: { CommandLogLine JSON }
//   - event: "done"  data: { task_id, exit_code }
//
// For already-completed tasks we replay stored logs then send done immediately.

pub async fn stream_command(
    Path(id): Path<String>,
    Extension(cmd_svc): Extension<Arc<CommandService>>,
    Extension(sse_hub): Extension<Arc<SseHub>>,
) -> impl IntoResponse {
    // Subscribe first (before reading history) to avoid race.
    let rx = sse_hub.subscribe(&id).await;

    // Check task existence
    let task = match cmd_svc.get_task(&id).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                e.log_and_user_message(),
            )
                .into_response();
        }
    };

    let is_terminal = matches!(
        task.status,
        CommandStatus::Success | CommandStatus::Failed | CommandStatus::Expired | CommandStatus::Timeout
    );

    // Gather historical logs (already stored).
    let historical = cmd_svc.get_task_logs(&id).await.unwrap_or_default();
    let task_id_clone = id.clone();
    let exit_code = task.exit_code;

    // Build an async stream of SSE events.
    let stream = async_stream::stream! {
        // Replay stored logs
        for line in &historical {
            yield Ok::<Event, Infallible>(SseHub::line_to_event(line));
        }

        if is_terminal {
            // Task already done: send done event and close stream.
            yield Ok(SseHub::done_event(&task_id_clone, exit_code));
            return;
        }

        // Forward live events from the broadcast receiver.
        let live = BroadcastStream::new(rx);
        tokio::pin!(live);
        while let Some(item) = live.next().await {
            match item {
                Ok(line) => yield Ok(SseHub::line_to_event(&line)),
                Err(_) => {
                    // Channel closed (task finished) → send done event.
                    // Re-fetch to get latest exit_code.
                    break;
                }
            }
        }

        yield Ok(SseHub::done_event(&task_id_clone, exit_code));
    };

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("ping"),
        )
        .into_response()
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::{TestAppBuilder, auth_headers};
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode, header},
    };
    use serde_json::json;
    use tower::ServiceExt;

    async fn make_get(app: &axum::Router, path: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::GET)
            .uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        (status, body)
    }

    async fn make_put(app: &axum::Router, path: &str, token: Option<&str>, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::PUT)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::from(serde_json::to_vec(&body).unwrap())).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        (status, body)
    }

    async fn make_post(app: &axum::Router, path: &str, token: Option<&str>, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::from(serde_json::to_vec(&body).unwrap())).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn test_get_config() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(&app.router, "/api/v1/remote-exec/config", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }

    #[tokio::test]
    async fn test_update_config() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_put(&app.router, "/api/v1/remote-exec/config", Some(&app.admin_token), json!({
            "enabled": true
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }

    #[tokio::test]
    async fn test_list_commands_empty() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(&app.router, "/api/v1/remote-exec/commands", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }
}
