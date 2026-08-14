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

use crate::middleware::permission::PermissionContext;
use crate::repository::client_repository::ClientRepository;
use crate::service::command_service::CommandService;
use crate::service::sse_hub::SseHub;
use axum::Json;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use common::command::CommandStatus;
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::entity::user::User;
use common::models::{
    ApiResponse, CommandTaskListResponse, CreateCommandRequest, UpdateRemoteExecConfigRequest,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub client_id: Option<String>,
}

// ── GET /api/v1/remote-exec/config ─────────────────────────────────────────

pub async fn get_config(Extension(cmd_svc): Extension<Arc<CommandService>>) -> impl IntoResponse {
    match cmd_svc.get_config().await {
        Ok(cfg) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "OK".into(),
                data: Some(cfg),
            }),
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
            Json(ApiResponse {
                status: 200,
                message: "OK".into(),
                data: Some(cfg),
            }),
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
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(req): Json<CreateCommandRequest>,
) -> impl IntoResponse {
    // Admin-only: require_admin middleware ensures only admins reach this handler.
    // allow_force = true so admins can override warnings, but blocked commands are never allowed.
    match cmd_svc
        .create_task(
            &req,
            &user.username,
            true,
            &user.id,
            &user.role,
            &perm_ctx.group_ids,
        )
        .await
    {
        Ok(resp) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "OK".into(),
                data: Some(resp),
            }),
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
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(user): Extension<User>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20).min(100);
    let scope = perm_ctx
        .evaluate(&ResourceType::Command, &PermissionAction::View)
        .unwrap_or(ScopeConstraint::None);
    if matches!(scope, ScopeConstraint::None) {
        return forbidden_response("you do not have permission to view command history");
    }
    let client_id = q.client_id.as_deref();
    let result = match &scope {
        ScopeConstraint::Project(_) | ScopeConstraint::Tag(_) => {
            let allowed_client_ids =
                match allowed_client_ids_for_scope(&client_repo, &perm_ctx, &scope).await {
                    Ok(ids) => ids,
                    Err(message) => return forbidden_response(&message),
                };
            cmd_svc
                .list_tasks_for_client_ids(&allowed_client_ids, client_id, page, page_size)
                .await
        }
        ScopeConstraint::All => cmd_svc.list_tasks(client_id, page, page_size).await,
        ScopeConstraint::Owned => {
            cmd_svc
                .list_tasks_for_submitter(client_id, &user.id, &user.username, page, page_size)
                .await
        }
        ScopeConstraint::None => unreachable!(),
    };
    match result {
        Ok((tasks, total)) => {
            let body = CommandTaskListResponse {
                tasks,
                total,
                page,
                page_size,
            };
            (
                StatusCode::OK,
                Json(ApiResponse {
                    status: 200,
                    message: "OK".into(),
                    data: Some(body),
                }),
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
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    match cmd_svc.get_task(&id).await {
        Ok(task) => {
            if !can_view_task(&perm_ctx, &task, Some(&user.username), &client_repo).await {
                return forbidden_response("you do not have permission to view this command");
            }
            (
                StatusCode::OK,
                Json(ApiResponse {
                    status: 200,
                    message: "OK".into(),
                    data: Some(task),
                }),
            )
                .into_response()
        }
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
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let task = match cmd_svc.get_task(&id).await {
        Ok(task) => task,
        Err(e) => {
            let code = e.status_code();
            return (
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<()> {
                    status: code,
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response();
        }
    };
    if !can_view_task(&perm_ctx, &task, Some(&user.username), &client_repo).await {
        return forbidden_response("you do not have permission to view this command");
    }
    match cmd_svc.get_task_logs(&id).await {
        Ok(logs) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "OK".into(),
                data: Some(logs),
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
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    // Check task existence
    let task = match cmd_svc.get_task(&id).await {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::NOT_FOUND, e.log_and_user_message()).into_response();
        }
    };
    if !can_view_task(&perm_ctx, &task, Some(&user.username), &client_repo).await {
        return forbidden_response("you do not have permission to view this command");
    }

    // Subscribe after authorization so an unauthorized caller cannot attach
    // to the live channel for another user's task.
    let rx = sse_hub.subscribe(&id).await;

    let is_terminal = matches!(
        task.status,
        CommandStatus::Success
            | CommandStatus::Failed
            | CommandStatus::Expired
            | CommandStatus::Timeout
    );

    // Gather historical logs (already stored).
    let historical = cmd_svc.get_task_logs(&id).await.unwrap_or_default();
    let task_id_clone = id.clone();
    let exit_code = task.exit_code;
    let cmd_svc_for_stream = cmd_svc.clone();

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

        let final_exit_code = cmd_svc_for_stream
            .get_task(&task_id_clone)
            .await
            .ok()
            .and_then(|task| task.exit_code)
            .or(exit_code);
        yield Ok(SseHub::done_event(&task_id_clone, final_exit_code));
    };

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("ping"),
        )
        .into_response()
}

async fn can_view_task(
    perm_ctx: &PermissionContext,
    task: &common::command::CommandTask,
    username: Option<&str>,
    client_repo: &ClientRepository,
) -> bool {
    if perm_ctx.is_admin() {
        return true;
    }
    match perm_ctx
        .evaluate(&ResourceType::Command, &PermissionAction::View)
        .unwrap_or(ScopeConstraint::None)
    {
        ScopeConstraint::All => true,
        ScopeConstraint::Owned => {
            task.submitted_by_id.as_deref() == Some(perm_ctx.user_id.as_str())
                || (task.submitted_by_id.is_none()
                    && username.is_some_and(|name| task.submitted_by == name))
        }
        ScopeConstraint::Project(_) | ScopeConstraint::Tag(_) => {
            let Some(client) = client_repo.get(&task.client_id).await.ok().flatten() else {
                return false;
            };
            PermissionContext::matches_scope(
                &perm_ctx
                    .evaluate(&ResourceType::Command, &PermissionAction::View)
                    .unwrap_or(ScopeConstraint::None),
                &perm_ctx.user_id,
                None,
                client.project_id.as_deref(),
                &PermissionContext::client_tags(&client),
            )
        }
        ScopeConstraint::None => false,
    }
}

async fn allowed_client_ids_for_scope(
    client_repo: &ClientRepository,
    perm_ctx: &PermissionContext,
    scope: &ScopeConstraint,
) -> Result<HashSet<String>, String> {
    let clients = client_repo
        .list_all()
        .await
        .map_err(|_| "Unable to resolve command target scope".to_string())?;
    Ok(clients
        .into_iter()
        .filter(|client| {
            PermissionContext::matches_scope(
                scope,
                &perm_ctx.user_id,
                None,
                client.project_id.as_deref(),
                &PermissionContext::client_tags(client),
            )
        })
        .map(|client| client.id)
        .collect())
}

fn forbidden_response(message: &str) -> axum::response::Response {
    (
        StatusCode::FORBIDDEN,
        Json(ApiResponse::<()> {
            status: 403,
            message: message.to_string(),
            data: None,
        }),
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

    async fn make_get(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::GET).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    async fn make_put(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::PUT)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    #[allow(dead_code)]
    async fn make_post(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn test_get_config() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(
            &app.router,
            "/api/v1/remote-exec/config",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }

    #[tokio::test]
    async fn test_update_config() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_put(
            &app.router,
            "/api/v1/remote-exec/config",
            Some(&app.admin_token),
            json!({
                "enabled": true
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }

    #[tokio::test]
    async fn test_list_commands_empty() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(
            &app.router,
            "/api/v1/remote-exec/commands",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], 200);
    }
}
