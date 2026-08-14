use std::cell::RefCell;
use std::rc::Rc;

use futures::lock::Mutex;
use gloo_net::http::{Request, Response};
use gloo_storage::{LocalStorage, Storage};
use log::{info, warn};
use serde::Serialize;
use web_sys::window;

use crate::stores::auth_store::AuthStore;
use crate::types::{ApiResponse, LoginResponse};
use common::entity::user::RefreshRequest;

const REFRESH_URL: &str = "/api/v1/auth/refresh";

thread_local! {
    static REFRESH_LOCK: RefCell<Rc<Mutex<()>>> = RefCell::new(Rc::new(Mutex::new(())));
}

fn load_auth_store() -> Option<AuthStore> {
    LocalStorage::get::<AuthStore>("auth_store")
        .ok()
        .or_else(|| LocalStorage::get::<AuthStore>("AuthStore").ok())
}

fn save_auth_store(store: &AuthStore) {
    let _ = LocalStorage::set("auth_store", store);
    let _ = LocalStorage::set("AuthStore", store);
    #[cfg(target_arch = "wasm32")]
    yewdux::Dispatch::<AuthStore>::global().set(store.clone());
}

pub fn clear_auth_and_redirect() {
    LocalStorage::delete("auth_store");
    LocalStorage::delete("AuthStore");
    #[cfg(target_arch = "wasm32")]
    yewdux::Dispatch::<AuthStore>::global().set(AuthStore::default());
    if let Some(win) = window() {
        let _ = win.location().set_href("/login");
    }
}

pub fn access_token() -> Option<String> {
    load_auth_store().and_then(|store| store.token)
}

pub fn auth_header() -> Option<String> {
    access_token().map(|token| format!("Bearer {}", token))
}

fn authorized_request(
    builder: gloo_net::http::RequestBuilder,
    token: Option<&str>,
) -> gloo_net::http::RequestBuilder {
    match token {
        Some(token) => builder.header("Authorization", &format!("Bearer {}", token)),
        None => builder,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RefreshResult {
    Refreshed,
    Unavailable,
    Rejected,
}

async fn refresh_after_unauthorized(failed_access_token: Option<&str>) -> RefreshResult {
    let lock = REFRESH_LOCK.with(|lock| lock.borrow().clone());
    let _guard = lock.lock().await;

    let Some(current) = load_auth_store() else {
        return RefreshResult::Rejected;
    };

    // Another request may already have rotated the token while this request
    // was waiting for the refresh lock.
    if current.token.as_deref() != failed_access_token && current.token.is_some() {
        return RefreshResult::Refreshed;
    }

    let Some(refresh_token) = current.refresh_token.clone() else {
        return RefreshResult::Rejected;
    };
    let request = RefreshRequest { refresh_token };
    let response = match Request::post(REFRESH_URL).json(&request) {
        Ok(request) => match request.send().await {
            Ok(response) => response,
            Err(error) => {
                warn!("Token refresh request failed: {}", error);
                return RefreshResult::Unavailable;
            }
        },
        Err(error) => {
            warn!("Failed to build token refresh request: {}", error);
            return RefreshResult::Unavailable;
        }
    };

    if response.status() == 401 || response.status() == 403 {
        // A different browser tab may have completed token rotation while
        // this refresh request was in flight.
        if access_token().as_deref() != failed_access_token && access_token().is_some() {
            return RefreshResult::Refreshed;
        }
        return RefreshResult::Rejected;
    }
    if response.status() != 200 {
        warn!("Token refresh returned HTTP {}", response.status());
        return RefreshResult::Unavailable;
    }

    match response.json::<ApiResponse<LoginResponse>>().await {
        Ok(payload) => match payload.data {
            Some(login) => {
                save_auth_store(&AuthStore {
                    token: Some(login.token),
                    refresh_token: Some(login.refresh_token),
                    user: Some(login.user),
                    is_authenticated: true,
                });
                info!("Access token refreshed successfully");
                RefreshResult::Refreshed
            }
            None => RefreshResult::Unavailable,
        },
        Err(error) => {
            warn!("Failed to parse token refresh response: {}", error);
            RefreshResult::Unavailable
        }
    }
}

async fn retry_decision(response: &Response, failed_token: Option<&str>) -> bool {
    if response.status() != 401 {
        return false;
    }
    match refresh_after_unauthorized(failed_token).await {
        RefreshResult::Refreshed => true,
        RefreshResult::Rejected => {
            clear_auth_and_redirect();
            false
        }
        RefreshResult::Unavailable => false,
    }
}

pub async fn get(url: &str) -> Result<Response, gloo_net::Error> {
    let token = access_token();
    let response = authorized_request(Request::get(url), token.as_deref())
        .send()
        .await?;
    if retry_decision(&response, token.as_deref()).await {
        let retry_token = access_token();
        let retry = authorized_request(Request::get(url), retry_token.as_deref())
            .send()
            .await?;
        if retry.status() == 401 {
            clear_auth_and_redirect();
        }
        return Ok(retry);
    }
    Ok(response)
}

pub async fn post_public_json<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
) -> Result<Response, gloo_net::Error> {
    Request::post(url).json(body)?.send().await
}

pub async fn post_json<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
) -> Result<Response, gloo_net::Error> {
    let token = access_token();
    let response = authorized_request(Request::post(url), token.as_deref())
        .json(body)?
        .send()
        .await?;
    if retry_decision(&response, token.as_deref()).await {
        let retry_token = access_token();
        let retry = authorized_request(Request::post(url), retry_token.as_deref())
            .json(body)?
            .send()
            .await?;
        if retry.status() == 401 {
            clear_auth_and_redirect();
        }
        return Ok(retry);
    }
    Ok(response)
}

pub async fn put_json<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
) -> Result<Response, gloo_net::Error> {
    let token = access_token();
    let response = authorized_request(Request::put(url), token.as_deref())
        .json(body)?
        .send()
        .await?;
    if retry_decision(&response, token.as_deref()).await {
        let retry_token = access_token();
        let retry = authorized_request(Request::put(url), retry_token.as_deref())
            .json(body)?
            .send()
            .await?;
        if retry.status() == 401 {
            clear_auth_and_redirect();
        }
        return Ok(retry);
    }
    Ok(response)
}

pub async fn delete(url: &str) -> Result<Response, gloo_net::Error> {
    let token = access_token();
    let response = authorized_request(Request::delete(url), token.as_deref())
        .send()
        .await?;
    if retry_decision(&response, token.as_deref()).await {
        let retry_token = access_token();
        let retry = authorized_request(Request::delete(url), retry_token.as_deref())
            .send()
            .await?;
        if retry.status() == 401 {
            clear_auth_and_redirect();
        }
        return Ok(retry);
    }
    Ok(response)
}
