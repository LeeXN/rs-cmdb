use crate::middleware::agent_auth::AuthenticatedAgent;
use crate::service::terminal_session_service::TerminalSessionService;
use axum::{
    Json,
    extract::{
        Extension, Path, Query, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::models::{
    AgentTerminalOutputRequest, AgentTerminalStateRequest, AgentTerminalStreamClientMessage,
    AgentTerminalStreamServerMessage, ApiResponse,
};
use futures::{SinkExt, StreamExt};
use serde_json::to_string;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

const LONG_POLL_HOLD: Duration = Duration::from_secs(30);
const LONG_POLL_INTERVAL: Duration = Duration::from_secs(2);
const STREAM_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

pub async fn stream_terminal(
    ws: WebSocketUpgrade,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(claim_id) = params
        .get("claim_id")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                status: 400,
                message: "claim_id query param required".into(),
                data: None,
            }),
        )
            .into_response();
    };

    ws.on_upgrade(move |socket| async move {
        handle_terminal_stream(socket, terminal_svc, agent.client_id, claim_id).await;
    })
}

pub async fn poll_pending(
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
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
    let session_id = params.get("session_id").map(String::as_str);
    let Some(claim_id) = params
        .get("claim_id")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                status: 400,
                message: "claim_id query param required".into(),
                data: None,
            }),
        )
            .into_response();
    };

    let start = tokio::time::Instant::now();
    loop {
        match terminal_svc
            .poll_for_agent(&client_id, session_id, Some(claim_id))
            .await
        {
            Ok(Some(session)) => {
                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        status: 200,
                        message: "OK".into(),
                        data: Some(session),
                    }),
                )
                    .into_response();
            }
            Ok(None) => {}
            Err(e) => {
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
            return StatusCode::NO_CONTENT.into_response();
        }
        sleep(LONG_POLL_INTERVAL).await;
    }
}

pub async fn push_output(
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
    Json(req): Json<AgentTerminalOutputRequest>,
) -> impl IntoResponse {
    match terminal_svc
        .report_output_for_client(&id, &agent.client_id, req)
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

pub async fn report_state(
    Path(id): Path<String>,
    Extension(terminal_svc): Extension<Arc<TerminalSessionService>>,
    Extension(agent): Extension<AuthenticatedAgent>,
    Json(req): Json<AgentTerminalStateRequest>,
) -> impl IntoResponse {
    match terminal_svc
        .report_state_for_client(&id, &agent.client_id, req)
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

async fn handle_terminal_stream(
    socket: WebSocket,
    terminal_svc: Arc<TerminalSessionService>,
    client_id: String,
    claim_id: String,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut client_events = terminal_svc.subscribe_client_events(&client_id).await;
    let mut heartbeat = tokio::time::interval(STREAM_HEARTBEAT_INTERVAL);

    if send_pending_work(&mut sender, &terminal_svc, &client_id, &claim_id)
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            incoming = receiver.next() => {
                let Some(result) = incoming else {
                    break;
                };
                match result {
                    Ok(Message::Text(text)) => {
                        if handle_client_message(&terminal_svc, &client_id, &claim_id, &text).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Ping(payload)) => {
                        if sender.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Pong(_)) => {}
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Binary(_)) => {}
                    Err(err) => {
                        warn!(client_id, error = %err, "agent terminal stream receive error");
                        break;
                    }
                }
            }
            event = client_events.recv() => {
                match event {
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        if send_pending_work(&mut sender, &terminal_svc, &client_id, &claim_id)
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = heartbeat.tick() => {
                match to_string(&AgentTerminalStreamServerMessage::Heartbeat) {
                    Ok(payload) => {
                        if sender.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        warn!(client_id, error = %err, "failed to serialize terminal heartbeat");
                        break;
                    }
                }
            }
        }
    }
}

async fn send_pending_work(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    terminal_svc: &TerminalSessionService,
    client_id: &str,
    claim_id: &str,
) -> Result<(), ()> {
    match terminal_svc
        .collect_agent_work(client_id, Some(claim_id))
        .await
    {
        Ok(work_items) => {
            for work in work_items {
                let payload = match to_string(&AgentTerminalStreamServerMessage::Sync { work }) {
                    Ok(payload) => payload,
                    Err(err) => {
                        warn!(client_id, error = %err, "failed to serialize terminal sync message");
                        return Err(());
                    }
                };
                if sender.send(Message::Text(payload.into())).await.is_err() {
                    return Err(());
                }
            }
            Ok(())
        }
        Err(err) => {
            warn!(client_id, error = %err, "failed to collect terminal work for agent stream");
            Err(())
        }
    }
}

async fn handle_client_message(
    terminal_svc: &TerminalSessionService,
    client_id: &str,
    stream_claim_id: &str,
    payload: &str,
) -> Result<(), ()> {
    let message = match serde_json::from_str::<AgentTerminalStreamClientMessage>(payload) {
        Ok(message) => message,
        Err(err) => {
            debug!(error = %err, "ignoring malformed terminal stream client message");
            return Ok(());
        }
    };

    let result = match message {
        AgentTerminalStreamClientMessage::Output {
            session_id,
            payload,
        } => {
            terminal_svc
                .report_output_for_client(&session_id, client_id, payload)
                .await
        }
        AgentTerminalStreamClientMessage::State {
            session_id,
            payload,
        } => {
            terminal_svc
                .report_state_for_client(&session_id, client_id, payload)
                .await
        }
        AgentTerminalStreamClientMessage::Heartbeat {
            session_ids,
            claim_id,
        } => {
            if claim_id != stream_claim_id {
                warn!(
                    client_id,
                    "agent terminal heartbeat claim does not match stream claim"
                );
                return Err(());
            }
            for session_id in session_ids {
                if let Err(err) = terminal_svc
                    .heartbeat_for_client(&session_id, client_id, &claim_id)
                    .await
                {
                    warn!(error = %err, "agent terminal heartbeat handling failed");
                    return Err(());
                }
            }
            return Ok(());
        }
    };

    if let Err(err) = result {
        warn!(error = %err, "agent terminal stream message handling failed");
        return Err(());
    }

    Ok(())
}
