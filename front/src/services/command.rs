//! API service for remote command execution.

use urlencoding;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::Callback;

use crate::services::auth;
use crate::types::ApiResponse;
use common::command::{CommandLogLine, CommandTask};
use common::entity::execution::ExecutionSession;
use common::models::{
    CastCleanupRequest, CastCleanupResponse, CommandTaskListResponse, CreateCommandRequest,
    CreateCommandResponse, CreateTerminalSessionRequest, CreateTerminalSessionResponse,
    PaginatedResult, RemoteExecConfigResponse, RemoteExecOpsOverviewResponse,
    ResizeTerminalRequest, TerminalInputRequest, TerminalOpsSummaryResponse,
    TerminalSessionSummary, UpdateRemoteExecConfigRequest,
};

const BASE: &str = "/api/v1/remote-exec";

pub fn get_auth_header() -> Option<String> {
    auth::auth_header()
}

pub fn get_access_token() -> Option<String> {
    get_auth_header().and_then(|value| value.strip_prefix("Bearer ").map(ToOwned::to_owned))
}

async fn get(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    auth::get(url).await
}

async fn post_json<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    auth::post_json(url, body).await
}

async fn put_json<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    auth::put_json(url, body).await
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

// ── config ──────────────────────────────────────────────────────────────────

pub async fn fetch_remote_exec_config() -> Result<RemoteExecConfigResponse, ApiError> {
    let url = format!("{}/config", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<RemoteExecConfigResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn update_remote_exec_config(
    enabled: bool,
) -> Result<RemoteExecConfigResponse, ApiError> {
    let url = format!("{}/config", BASE);
    let body = UpdateRemoteExecConfigRequest { enabled };
    match put_json(&url, &body).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<RemoteExecConfigResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

// ── commands ─────────────────────────────────────────────────────────────────

pub async fn create_command(req: &CreateCommandRequest) -> Result<CreateCommandResponse, ApiError> {
    let url = format!("{}/commands", BASE);
    match post_json(&url, req).await {
        Ok(resp) if resp.status() == 200 || resp.status() == 201 => resp
            .json::<ApiResponse<CreateCommandResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => {
            // Extract message from body if possible
            let msg = resp
                .json::<ApiResponse<serde_json::Value>>()
                .await
                .map(|r| r.message)
                .unwrap_or_else(|_| "Command creation failed".into());
            Err(ApiError { message: msg })
        }
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_commands(
    client_id: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<CommandTaskListResponse, ApiError> {
    let mut url = format!("{}/commands?page={}&page_size={}", BASE, page, page_size);
    if let Some(cid) = client_id {
        url.push_str(&format!("&client_id={}", cid));
    }
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<CommandTaskListResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_command(task_id: &str) -> Result<CommandTask, ApiError> {
    let url = format!("{}/commands/{}", BASE, task_id);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<CommandTask>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_command_logs(task_id: &str) -> Result<Vec<CommandLogLine>, ApiError> {
    let url = format!("{}/commands/{}/logs", BASE, task_id);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<Vec<CommandLogLine>>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

// ── Execution Sessions ──────────────────────────────────────────────────────────────────

const SESSION_BASE: &str = "/api/v1/sessions";
const TERMINAL_BASE: &str = "/api/v1/remote-exec/terminal-sessions";

#[allow(clippy::too_many_arguments)]
pub async fn fetch_sessions(
    search: Option<&str>,
    client_id: Option<&str>,
    user_id: Option<&str>,
    status: Option<&str>,
    execution_type: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<PaginatedResult<ExecutionSession>, ApiError> {
    let mut url = format!("{}?page={}&page_size={}", SESSION_BASE, page, page_size);
    if let Some(search_term) = search {
        if !search_term.is_empty() {
            url.push_str(&format!("&search={}", urlencoding::encode(search_term)));
        }
    }
    if let Some(cid) = client_id {
        if !cid.is_empty() {
            url.push_str(&format!("&client_id={}", urlencoding::encode(cid)));
        }
    }
    if let Some(uid) = user_id {
        if !uid.is_empty() {
            url.push_str(&format!("&user_id={}", urlencoding::encode(uid)));
        }
    }
    if let Some(s) = status {
        if !s.is_empty() && s != "all" {
            url.push_str(&format!("&status={}", urlencoding::encode(s)));
        }
    }
    if let Some(kind) = execution_type {
        if !kind.is_empty() && kind != "all" {
            url.push_str(&format!("&execution_type={}", urlencoding::encode(kind)));
        }
    }
    if let Some(start) = from {
        if !start.is_empty() {
            url.push_str(&format!("&from={}", urlencoding::encode(start)));
        }
    }
    if let Some(end) = to {
        if !end.is_empty() {
            url.push_str(&format!("&to={}", urlencoding::encode(end)));
        }
    }
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<PaginatedResult<ExecutionSession>>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_session(session_id: &str) -> Result<ExecutionSession, ApiError> {
    let url = format!("{}/{}", SESSION_BASE, session_id);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<ExecutionSession>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_session_cast(session_id: &str) -> Result<Vec<u8>, ApiError> {
    let url = format!("{}/{}/cast", SESSION_BASE, session_id);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp.binary().await.map_err(|e| ApiError {
            message: e.to_string(),
        }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn create_terminal_session(
    req: &CreateTerminalSessionRequest,
) -> Result<CreateTerminalSessionResponse, ApiError> {
    match post_json(TERMINAL_BASE, req).await {
        Ok(resp) if resp.status() == 200 || resp.status() == 201 => resp
            .json::<ApiResponse<CreateTerminalSessionResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_terminal_session(session_id: &str) -> Result<TerminalSessionSummary, ApiError> {
    let url = format!("{}/{}", TERMINAL_BASE, session_id);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<TerminalSessionSummary>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn list_terminal_sessions(
    client_id: &str,
) -> Result<Vec<TerminalSessionSummary>, ApiError> {
    let url = format!(
        "{}?client_id={}",
        TERMINAL_BASE,
        urlencoding::encode(client_id)
    );
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<Vec<TerminalSessionSummary>>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn push_terminal_input(
    session_id: &str,
    input: &TerminalInputRequest,
) -> Result<(), ApiError> {
    let url = format!("{}/{}/input", TERMINAL_BASE, session_id);
    match post_json(&url, input).await {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn resize_terminal_session(
    session_id: &str,
    req: &ResizeTerminalRequest,
) -> Result<(), ApiError> {
    let url = format!("{}/{}/resize", TERMINAL_BASE, session_id);
    match post_json(&url, req).await {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_remote_exec_ops_overview() -> Result<RemoteExecOpsOverviewResponse, ApiError> {
    let url = format!("{}/ops/overview", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<RemoteExecOpsOverviewResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_terminal_ops_summary() -> Result<TerminalOpsSummaryResponse, ApiError> {
    let url = format!("{}/terminal-ops/summary", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<TerminalOpsSummaryResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn list_all_terminal_sessions() -> Result<Vec<TerminalSessionSummary>, ApiError> {
    let url = format!("{}/terminal-sessions/all", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<Vec<TerminalSessionSummary>>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn cleanup_cast_history(retention_days: u64) -> Result<CastCleanupResponse, ApiError> {
    let url = format!("{}/casts/cleanup", BASE);
    let body = CastCleanupRequest { retention_days };
    match post_json(&url, &body).await {
        Ok(resp) if resp.status() == 200 => resp
            .json::<ApiResponse<CastCleanupResponse>>()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
            })
            .and_then(|r| {
                r.data.ok_or(ApiError {
                    message: "no data".into(),
                })
            }),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn close_terminal_session(session_id: &str) -> Result<(), ApiError> {
    let url = format!("{}/{}", TERMINAL_BASE, session_id);
    match auth::delete(&url).await {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub fn terminal_ws_url(session_id: &str) -> Result<String, ApiError> {
    let window = window().ok_or(ApiError {
        message: "window unavailable".into(),
    })?;
    let location = window.location();
    let protocol = location.protocol().map_err(|_| ApiError {
        message: "protocol unavailable".into(),
    })?;
    let host = location.host().map_err(|_| ApiError {
        message: "host unavailable".into(),
    })?;
    let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };
    let token = get_access_token().ok_or(ApiError {
        message: "missing access token".into(),
    })?;
    Ok(format!(
        "{}://{}/api/v1/remote-exec/terminal-sessions/{}/ws?access_token={}",
        ws_protocol,
        host,
        session_id,
        urlencoding::encode(&token)
    ))
}

// ── Callback wrappers ────────────────────────────────────────────────────────

pub fn get_remote_exec_config(cb: Callback<Result<RemoteExecConfigResponse, ApiError>>) {
    spawn_local(async move { cb.emit(fetch_remote_exec_config().await) });
}

pub fn set_remote_exec_config(
    enabled: bool,
    cb: Callback<Result<RemoteExecConfigResponse, ApiError>>,
) {
    spawn_local(async move { cb.emit(update_remote_exec_config(enabled).await) });
}

pub fn submit_command(
    req: CreateCommandRequest,
    cb: Callback<Result<CreateCommandResponse, ApiError>>,
) {
    spawn_local(async move { cb.emit(create_command(&req).await) });
}

pub fn list_commands(
    client_id: Option<String>,
    page: usize,
    page_size: usize,
    cb: Callback<Result<CommandTaskListResponse, ApiError>>,
) {
    spawn_local(
        async move { cb.emit(fetch_commands(client_id.as_deref(), page, page_size).await) },
    );
}

pub fn get_command_logs(task_id: String, cb: Callback<Result<Vec<CommandLogLine>, ApiError>>) {
    spawn_local(async move { cb.emit(fetch_command_logs(&task_id).await) });
}

pub fn get_command(task_id: String, cb: Callback<Result<CommandTask, ApiError>>) {
    spawn_local(async move { cb.emit(fetch_command(&task_id).await) });
}
