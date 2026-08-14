//! API service for permission management (exec policies, web terminal policies, approvals).

use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use crate::services::auth;
use crate::types::{ApiResponse, ApprovalSummaryResponse, PermissionOverviewResponse};

const BASE: &str = "/api/v1/permissions";

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

async fn delete(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    auth::delete(url).await
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

async fn parse_json<T>(resp: gloo_net::http::Response) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned + PartialEq,
{
    resp.json::<ApiResponse<T>>()
        .await
        .map_err(|e| ApiError {
            message: e.to_string(),
        })
        .and_then(|r| r.data.ok_or(ApiError { message: r.message }))
}

async fn parse_error_message(resp: gloo_net::http::Response, fallback: &str) -> String {
    resp.json::<ApiResponse<serde_json::Value>>()
        .await
        .map(|r| r.message)
        .unwrap_or_else(|_| fallback.into())
}

// ── ExecPolicy ──────────────────────────────────────────────────────────────

pub async fn fetch_exec_policies() -> Result<Vec<serde_json::Value>, ApiError> {
    let url = format!("{}/exec-policies", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn create_exec_policy(policy: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/exec-policies", BASE);
    match post_json(&url, policy).await {
        Ok(resp) if resp.status() == 200 || resp.status() == 201 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Create failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn update_exec_policy(
    id: &str,
    policy: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/exec-policies/{}", BASE, id);
    match put_json(&url, policy).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn delete_exec_policy(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/exec-policies/{}", BASE, id);
    match delete(&url).await {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

// ── WebTerminalPolicy ───────────────────────────────────────────────────────

pub async fn fetch_web_terminal_policies() -> Result<Vec<serde_json::Value>, ApiError> {
    let url = format!("{}/web-terminal-policies", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn create_web_terminal_policy(
    policy: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/web-terminal-policies", BASE);
    match post_json(&url, policy).await {
        Ok(resp) if resp.status() == 200 || resp.status() == 201 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Create failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn update_web_terminal_policy(
    id: &str,
    policy: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/web-terminal-policies/{}", BASE, id);
    match put_json(&url, policy).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn delete_web_terminal_policy(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/web-terminal-policies/{}", BASE, id);
    match delete(&url).await {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

// ── Approvals ───────────────────────────────────────────────────────────────

pub async fn list_pending_approvals() -> Result<Vec<serde_json::Value>, ApiError> {
    let url = format!("{}/pending-approvals", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn list_my_approvals() -> Result<Vec<serde_json::Value>, ApiError> {
    let url = format!("{}/my-approvals", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn approve_request(id: &str) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/pending-approvals/{}/approve", BASE, id);
    match post_json(&url, &serde_json::json!({})).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn reject_request(
    id: &str,
    reason: Option<String>,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/pending-approvals/{}/reject", BASE, id);
    let body = serde_json::json!({ "reason": reason.unwrap_or_default() });
    match post_json(&url, &body).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_permission_overview() -> Result<PermissionOverviewResponse, ApiError> {
    let url = format!("{}/overview", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_permission_rules() -> Result<Vec<serde_json::Value>, ApiError> {
    let url = format!("{}/rules", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn create_permission_rule(
    rule: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/rules", BASE);
    match post_json(&url, rule).await {
        Ok(resp) if resp.status() == 200 || resp.status() == 201 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Create failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn update_permission_rule(
    id: &str,
    rule: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/rules/{}", BASE, id);
    match put_json(&url, rule).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Update failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn delete_permission_rule(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/rules/{}", BASE, id);
    match delete(&url).await {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Delete failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_permission_groups() -> Result<Vec<serde_json::Value>, ApiError> {
    let url = format!("{}/groups", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn create_permission_group(
    group: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/groups", BASE);
    match post_json(&url, group).await {
        Ok(resp) if resp.status() == 200 || resp.status() == 201 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Create failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn update_permission_group(
    id: &str,
    group: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}/groups/{}", BASE, id);
    match put_json(&url, group).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Update failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn delete_permission_group(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/groups/{}", BASE, id);
    match delete(&url).await {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(ApiError {
            message: parse_error_message(resp, "Delete failed").await,
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

pub async fn fetch_approval_summary() -> Result<ApprovalSummaryResponse, ApiError> {
    let url = format!("{}/approval-summary", BASE);
    match get(&url).await {
        Ok(resp) if resp.status() == 200 => parse_json(resp).await,
        Ok(resp) => Err(ApiError {
            message: format!("HTTP {}", resp.status()),
        }),
        Err(e) => Err(ApiError {
            message: e.to_string(),
        }),
    }
}

// ── Callback wrappers ───────────────────────────────────────────────────────

pub fn fetch_exec_policies_cb(cb: Callback<Result<Vec<serde_json::Value>, ApiError>>) {
    spawn_local(async move { cb.emit(fetch_exec_policies().await) });
}

pub fn create_exec_policy_cb(
    policy: serde_json::Value,
    cb: Callback<Result<serde_json::Value, ApiError>>,
) {
    spawn_local(async move { cb.emit(create_exec_policy(&policy).await) });
}

pub fn update_exec_policy_cb(
    id: String,
    policy: serde_json::Value,
    cb: Callback<Result<serde_json::Value, ApiError>>,
) {
    spawn_local(async move { cb.emit(update_exec_policy(&id, &policy).await) });
}

pub fn delete_exec_policy_cb(id: String, cb: Callback<Result<(), ApiError>>) {
    spawn_local(async move { cb.emit(delete_exec_policy(&id).await) });
}

pub fn fetch_web_terminal_policies_cb(cb: Callback<Result<Vec<serde_json::Value>, ApiError>>) {
    spawn_local(async move { cb.emit(fetch_web_terminal_policies().await) });
}

pub fn create_web_terminal_policy_cb(
    policy: serde_json::Value,
    cb: Callback<Result<serde_json::Value, ApiError>>,
) {
    spawn_local(async move { cb.emit(create_web_terminal_policy(&policy).await) });
}

pub fn update_web_terminal_policy_cb(
    id: String,
    policy: serde_json::Value,
    cb: Callback<Result<serde_json::Value, ApiError>>,
) {
    spawn_local(async move { cb.emit(update_web_terminal_policy(&id, &policy).await) });
}

pub fn delete_web_terminal_policy_cb(id: String, cb: Callback<Result<(), ApiError>>) {
    spawn_local(async move { cb.emit(delete_web_terminal_policy(&id).await) });
}

pub fn list_pending_approvals_cb(cb: Callback<Result<Vec<serde_json::Value>, ApiError>>) {
    spawn_local(async move { cb.emit(list_pending_approvals().await) });
}

pub fn list_my_approvals_cb(cb: Callback<Result<Vec<serde_json::Value>, ApiError>>) {
    spawn_local(async move { cb.emit(list_my_approvals().await) });
}

pub fn approve_request_cb(id: String, cb: Callback<Result<serde_json::Value, ApiError>>) {
    spawn_local(async move { cb.emit(approve_request(&id).await) });
}

pub fn reject_request_cb(
    id: String,
    reason: Option<String>,
    cb: Callback<Result<serde_json::Value, ApiError>>,
) {
    spawn_local(async move { cb.emit(reject_request(&id, reason).await) });
}
