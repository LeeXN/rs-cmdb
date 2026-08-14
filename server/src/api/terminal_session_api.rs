use crate::middleware::permission::PermissionContext;
use crate::service::terminal_session_service::TerminalSessionService;
use axum::{
    Extension as AxumExtension, Json,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
};
use common::entity::permission::PermissionAction;
use common::models::{
    ApiResponse, CreateTerminalSessionRequest, CreateTerminalSessionResponse,
    ResizeTerminalRequest, TerminalInputRequest,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;

pub async fn create_session(
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(req): Json<CreateTerminalSessionRequest>,
) -> impl IntoResponse {
    if !terminal_svc
        .allows_client_permission_scope(&perm_ctx, &req.client_id, &PermissionAction::Update)
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()> {
                status: 403,
                message: "Forbidden".into(),
                data: None,
            }),
        )
            .into_response();
    }
    match terminal_svc
        .create_session_with_groups(
            &req,
            &perm_ctx.user_id,
            &perm_ctx.user_id,
            &perm_ctx.role,
            &perm_ctx.group_ids,
        )
        .await
    {
        Ok(session) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "OK".into(),
                data: Some(CreateTerminalSessionResponse { session }),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }),
        )
            .into_response(),
    }
}

pub async fn get_session(
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match terminal_svc.get_session(&id).await {
        Ok(session) => {
            if !can_access_session(&terminal_svc, &perm_ctx, &session, &PermissionAction::View)
                .await
            {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<()> {
                        status: 403,
                        message: "Forbidden".into(),
                        data: None,
                    }),
                )
                    .into_response();
            }
            (
                StatusCode::OK,
                Json(ApiResponse {
                    status: 200,
                    message: "OK".into(),
                    data: Some(session),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }),
        )
            .into_response(),
    }
}

pub async fn list_sessions(
    Query(params): Query<HashMap<String, String>>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    let Some(client_id) = params.get("client_id") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                status: 400,
                message: "missing client_id".into(),
                data: None,
            }),
        )
            .into_response();
    };
    if !terminal_svc
        .allows_client_permission_scope(&perm_ctx, client_id, &PermissionAction::View)
        .await
    {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()> {
                status: 403,
                message: "Forbidden".into(),
                data: None,
            }),
        )
            .into_response();
    }
    match terminal_svc
        .list_client_sessions(client_id, &perm_ctx.user_id, perm_ctx.is_admin())
        .await
    {
        Ok(sessions) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "OK".into(),
                data: Some(sessions),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }),
        )
            .into_response(),
    }
}

pub async fn push_input(
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(req): Json<TerminalInputRequest>,
) -> impl IntoResponse {
    match terminal_svc.get_session(&id).await {
        Ok(session)
            if !can_access_session(
                &terminal_svc,
                &perm_ctx,
                &session,
                &PermissionAction::Update,
            )
            .await =>
        {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()> {
                    status: 403,
                    message: "Forbidden".into(),
                    data: None,
                }),
            )
                .into_response();
        }
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<()> {
                    status: e.status_code(),
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response();
        }
    }
    match terminal_svc.queue_input(&id, req.input).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                status: 200,
                message: "OK".into(),
                data: None,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }),
        )
            .into_response(),
    }
}

pub async fn resize_session(
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(req): Json<ResizeTerminalRequest>,
) -> impl IntoResponse {
    match terminal_svc.get_session(&id).await {
        Ok(session)
            if !can_access_session(
                &terminal_svc,
                &perm_ctx,
                &session,
                &PermissionAction::Update,
            )
            .await =>
        {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()> {
                    status: 403,
                    message: "Forbidden".into(),
                    data: None,
                }),
            )
                .into_response();
        }
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<()> {
                    status: e.status_code(),
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response();
        }
    }
    match terminal_svc.resize(&id, req.cols, req.rows).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                status: 200,
                message: "OK".into(),
                data: None,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }),
        )
            .into_response(),
    }
}

pub async fn close_session(
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match terminal_svc.get_session(&id).await {
        Ok(session)
            if !can_access_session(
                &terminal_svc,
                &perm_ctx,
                &session,
                &PermissionAction::Update,
            )
            .await =>
        {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()> {
                    status: 403,
                    message: "Forbidden".into(),
                    data: None,
                }),
            )
                .into_response();
        }
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<()> {
                    status: e.status_code(),
                    message: e.log_and_user_message(),
                    data: None,
                }),
            )
                .into_response();
        }
    }
    match terminal_svc
        .close_session(&id, Some("closed by operator".to_string()))
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
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<()> {
                status: e.status_code(),
                message: e.log_and_user_message(),
                data: None,
            }),
        )
            .into_response(),
    }
}

pub async fn stream_session(
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match terminal_svc.get_session(&id).await {
        Ok(session)
            if !can_access_session(&terminal_svc, &perm_ctx, &session, &PermissionAction::View)
                .await =>
        {
            return StatusCode::FORBIDDEN.into_response();
        }
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                e.log_and_user_message(),
            )
                .into_response();
        }
    }
    let rx = terminal_svc.subscribe_output(&id).await;
    let historical = match terminal_svc.get_output(&id).await {
        Ok(chunks) => chunks,
        Err(e) => return (StatusCode::NOT_FOUND, e.log_and_user_message()).into_response(),
    };
    let stream = async_stream::stream! {
        for chunk in historical {
            let data = serde_json::to_string(&chunk).unwrap_or_default();
            yield Ok::<Event, Infallible>(Event::default().event("output").data(data));
        }
        let mut broadcast = BroadcastStream::new(rx);
        while let Some(item) = broadcast.next().await {
            match item {
                Ok(chunk) => {
                    let data = serde_json::to_string(chunk.as_ref()).unwrap_or_default();
                    yield Ok::<Event, Infallible>(Event::default().event("output").data(data));
                }
                Err(_) => break,
            }
        }
    };
    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}

async fn can_access_session(
    terminal_svc: &TerminalSessionService,
    perm_ctx: &PermissionContext,
    session: &common::models::TerminalSessionSummary,
    action: &PermissionAction,
) -> bool {
    (perm_ctx.is_admin() || session.user_id == perm_ctx.user_id)
        && terminal_svc
            .allows_client_permission_scope(perm_ctx, &session.client_id, action)
            .await
}

pub async fn websocket_session(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    AxumExtension(perm_ctx): AxumExtension<PermissionContext>,
) -> Response {
    match terminal_svc.get_session(&id).await {
        Ok(session) => {
            if !can_access_session(
                &terminal_svc,
                &perm_ctx,
                &session,
                &PermissionAction::Update,
            )
            .await
            {
                return StatusCode::FORBIDDEN.into_response();
            }
            ws.on_upgrade(move |socket| handle_terminal_socket(socket, terminal_svc, session))
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn handle_terminal_socket(
    socket: WebSocket,
    terminal_svc: Arc<TerminalSessionService>,
    session: common::models::TerminalSessionSummary,
) {
    let (mut sender, mut receiver) = socket.split();

    if let Ok(historical) = terminal_svc.get_output(&session.session_id).await {
        for chunk in historical {
            if sender.send(Message::Text(chunk.data.into())).await.is_err() {
                return;
            }
        }
    }

    let mut output_rx = terminal_svc.subscribe_output(&session.session_id).await;
    let session_id = session.session_id.clone();
    let mut pending_line = String::new();
    let output_task = tokio::spawn(async move {
        loop {
            match output_rx.recv().await {
                Ok(chunk) => {
                    if sender
                        .send(Message::Text(chunk.data.clone().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });

    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Text(input)) => {
                if terminal_svc
                    .queue_browser_input(&session_id, input.to_string(), &mut pending_line)
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(Message::Binary(_)) => {}
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
            Err(_) => break,
        }
    }

    output_task.abort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::permission_repository::PermissionRepository;
    use crate::repository::terminal_session_repository::TerminalSessionRepository;
    use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
    use crate::service::permission_service::PermissionService;
    use crate::service::web_terminal_service::WebTerminalService;
    use async_trait::async_trait;
    use axum::body::to_bytes;
    use common::entity::permission::{
        CommandRules, SubjectType, TargetScope, TerminalMode, WebTerminalPolicy,
    };
    use common::entity::user::Role;
    use common::error::CmdbResult;
    use common::models::{
        ApiResponse, CreateTerminalSessionRequest, TerminalSessionState, TerminalSessionSummary,
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MemoryDb {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    #[async_trait]
    impl Database for MemoryDb {
        async fn set(&self, key: &str, value: &[u8]) -> CmdbResult<()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn get(&self, key: &str) -> CmdbResult<Option<Vec<u8>>> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        async fn delete(&self, key: &str) -> CmdbResult<()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }

        async fn list_keys(&self, prefix: &str) -> CmdbResult<Vec<String>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .keys()
                .filter(|key| key.starts_with(prefix))
                .cloned()
                .collect())
        }

        async fn list_values(&self, prefix: &str) -> CmdbResult<Vec<Vec<u8>>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .map(|(_, value)| value.clone())
                .collect())
        }

        async fn list_entries(&self, prefix: &str) -> CmdbResult<Vec<(String, Vec<u8>)>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect())
        }

        async fn exists(&self, key: &str) -> CmdbResult<bool> {
            Ok(self.data.lock().unwrap().contains_key(key))
        }

        async fn update_all(
            &self,
            prefix: &str,
            callback: Box<dyn Fn(String, Vec<u8>) -> Option<Vec<u8>> + Send + Sync>,
        ) -> CmdbResult<()> {
            let snapshot = self.data.lock().unwrap().clone();
            let mut updates = Vec::new();
            for (key, value) in snapshot {
                if key.starts_with(prefix) {
                    if let Some(new_value) = callback(key.clone(), value) {
                        updates.push((key, new_value));
                    }
                }
            }
            let mut data = self.data.lock().unwrap();
            for (key, value) in updates {
                data.insert(key, value);
            }
            Ok(())
        }
    }

    async fn build_terminal_service() -> Arc<TerminalSessionService> {
        let db: Arc<dyn Database> = Arc::new(MemoryDb::default());
        let repo = Arc::new(TerminalSessionRepository::new(db.clone()));
        let policy_repo = Arc::new(WebTerminalPolicyRepository::new(db));
        policy_repo
            .save(&WebTerminalPolicy {
                id: "policy-1".into(),
                name: "default".into(),
                description: "default test policy".into(),
                subject_type: SubjectType::Role,
                subject_id: "User".into(),
                target_scope: TargetScope::All,
                mode: TerminalMode::ReadWrite,
                terminal_command_rules: CommandRules::default(),
                allowed_commands: vec![],
                session_timeout_secs: 300,
                max_concurrent_sessions: 5,
                require_approval: false,
                priority: 1,
            })
            .await
            .unwrap();
        let policy = Arc::new(WebTerminalService::new(policy_repo));
        TerminalSessionService::new(repo, policy)
    }

    fn build_permission_context(user_id: &str, role: Role) -> PermissionContext {
        let db: Arc<dyn Database> = Arc::new(MemoryDb::default());
        let repo = Arc::new(PermissionRepository::new(db));
        let permission_service = Arc::new(PermissionService::new(repo));
        PermissionContext {
            user_id: user_id.to_string(),
            role,
            group_ids: Vec::new(),
            permission_service,
        }
    }

    async fn response_json<T>(response: axum::response::Response) -> ApiResponse<T>
    where
        T: serde::de::DeserializeOwned + PartialEq,
    {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn seed_session(
        svc: &Arc<TerminalSessionService>,
        client_id: &str,
        user_id: &str,
        shell: &str,
        state: TerminalSessionState,
    ) -> TerminalSessionSummary {
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: client_id.to_string(),
                    shell: Some(shell.to_string()),
                    cols: Some(80),
                    rows: Some(24),
                },
                user_id,
                user_id,
                &Role::User,
            )
            .await
            .unwrap();
        if state == TerminalSessionState::Active {
            svc.report_state(
                &session.session_id,
                common::models::AgentTerminalStateRequest {
                    claim_id: Some(format!("claim-{}", session.session_id)),
                    state,
                    message: None,
                },
            )
            .await
            .unwrap();
        }
        svc.get_session(&session.session_id).await.unwrap()
    }

    #[tokio::test]
    async fn list_sessions_requires_client_id_query() {
        let response = list_sessions(
            Query(HashMap::new()),
            Extension(build_terminal_service().await),
            Extension(build_permission_context("user-a", Role::User)),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json::<()>(response).await;
        assert_eq!(body.status, 400);
        assert_eq!(body.message, "missing client_id");
        assert_eq!(body.data, None);
    }

    #[tokio::test]
    async fn list_sessions_respects_visibility_and_returns_sorted_sessions() {
        let svc = build_terminal_service().await;

        let own_pending = seed_session(
            &svc,
            "client-api-list",
            "user-a",
            "/bin/bash",
            TerminalSessionState::Pending,
        )
        .await;
        let other_active = seed_session(
            &svc,
            "client-api-list",
            "user-b",
            "/bin/sh",
            TerminalSessionState::Active,
        )
        .await;

        let user_response = list_sessions(
            Query(HashMap::from([(
                "client_id".to_string(),
                "client-api-list".to_string(),
            )])),
            Extension(svc.clone()),
            Extension(build_permission_context("user-a", Role::User)),
        )
        .await
        .into_response();

        assert_eq!(user_response.status(), StatusCode::OK);
        let user_body = response_json::<Vec<TerminalSessionSummary>>(user_response).await;
        let user_sessions = user_body.data.unwrap();
        assert_eq!(
            user_sessions
                .iter()
                .map(|session| session.session_id.as_str())
                .collect::<Vec<_>>(),
            vec![own_pending.session_id.as_str()]
        );

        let admin_response = list_sessions(
            Query(HashMap::from([(
                "client_id".to_string(),
                "client-api-list".to_string(),
            )])),
            Extension(svc),
            Extension(build_permission_context("admin-1", Role::Admin)),
        )
        .await
        .into_response();

        assert_eq!(admin_response.status(), StatusCode::OK);
        let admin_body = response_json::<Vec<TerminalSessionSummary>>(admin_response).await;
        let admin_sessions = admin_body.data.unwrap();
        assert_eq!(
            admin_sessions
                .iter()
                .map(|session| session.session_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                other_active.session_id.as_str(),
                own_pending.session_id.as_str(),
            ]
        );
    }
}
