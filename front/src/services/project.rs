use gloo_net::http::Request;
use log::info;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;
use gloo_storage::{LocalStorage, Storage};
use web_sys::window;

use common::models::{Project, PaginatedResult};
use crate::types::ApiResponse;
use crate::stores::auth_store::AuthStore;
use urlencoding;

const API_BASE_URL: &str = "/api/v1/projects";

fn get_auth_header() -> Option<String> {
    if let Ok(store) = LocalStorage::get::<AuthStore>("auth_store") {
        if let Some(token) = store.token {
            return Some(format!("Bearer {}", token));
        }
    }
    if let Ok(store) = LocalStorage::get::<AuthStore>("AuthStore") {
        if let Some(token) = store.token {
            return Some(format!("Bearer {}", token));
        }
    }
    None
}

async fn request_get(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::get(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.send().await?;
    check_auth_error(&response);
    Ok(response)
}

async fn request_post<T: serde::Serialize>(url: &str, body: &T) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::post(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.json(body)?.send().await?;
    check_auth_error(&response);
    Ok(response)
}

async fn request_put<T: serde::Serialize>(url: &str, body: &T) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::put(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.json(body)?.send().await?;
    check_auth_error(&response);
    Ok(response)
}

async fn request_delete(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::delete(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.send().await?;
    check_auth_error(&response);
    Ok(response)
}

fn check_auth_error(response: &gloo_net::http::Response) {
    if response.status() == 401 {
        info!("Received 401 Unauthorized, redirecting to login...");
        let _ = LocalStorage::delete("auth_store");
        let _ = LocalStorage::delete("AuthStore");
        if let Some(win) = window() {
            let _ = win.location().set_href("/login");
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

pub async fn fetch_projects(page: usize, page_size: usize, search: Option<String>, department: Option<String>) -> Result<PaginatedResult<Project>, ApiError> {
    let mut url = format!("{}?page={}&page_size={}", API_BASE_URL, page, page_size);
    
    if let Some(s) = search {
        if !s.is_empty() {
            url.push_str(&format!("&search={}", urlencoding::encode(&s)));
        }
    }
    
    if let Some(d) = department {
        if !d.is_empty() {
            url.push_str(&format!("&department={}", urlencoding::encode(&d)));
        }
    }

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<PaginatedResult<Project>>>().await {
                    Ok(data) => {
                        if let Some(result) = data.data {
                            Ok(result)
                        } else {
                            Err(ApiError { message: "No data returned".to_string() })
                        }
                    },
                    Err(err) => Err(ApiError { message: format!("Failed to parse projects: {}", err) }),
                }
            } else {
                Err(ApiError { message: format!("Failed to fetch projects: HTTP {}", response.status()) })
            }
        },
        Err(err) => Err(ApiError { message: format!("Network error: {}", err) }),
    }
}

pub async fn create_project(project: &Project) -> Result<Project, ApiError> {
    match request_post(API_BASE_URL, project).await {
        Ok(response) => {
            if response.status() == 201 {
                match response.json::<ApiResponse<Project>>().await {
                    Ok(data) => Ok(data.data.ok_or(ApiError { message: "No data returned".to_string() })?),
                    Err(err) => Err(ApiError { message: format!("Failed to parse created project: {}", err) }),
                }
            } else {
                // Try to parse error message
                let error_msg = match response.json::<ApiResponse<Project>>().await {
                    Ok(data) => data.message,
                    Err(_) => format!("Failed to create project: HTTP {}", response.status()),
                };
                Err(ApiError { message: error_msg })
            }
        },
        Err(err) => Err(ApiError { message: format!("Network error: {}", err) }),
    }
}

pub async fn update_project(id: &str, project: &Project) -> Result<Project, ApiError> {
    let url = format!("{}/{}", API_BASE_URL, id);
    match request_put(&url, project).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Project>>().await {
                    Ok(data) => Ok(data.data.ok_or(ApiError { message: "No data returned".to_string() })?),
                    Err(err) => Err(ApiError { message: format!("Failed to parse updated project: {}", err) }),
                }
            } else {
                // Try to parse error message
                let error_msg = match response.json::<ApiResponse<Project>>().await {
                    Ok(data) => data.message,
                    Err(_) => format!("Failed to update project: HTTP {}", response.status()),
                };
                Err(ApiError { message: error_msg })
            }
        },
        Err(err) => Err(ApiError { message: format!("Network error: {}", err) }),
    }
}

pub async fn delete_project(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/{}", API_BASE_URL, id);
    match request_delete(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                Ok(())
            } else {
                // Try to parse error message from ApiResponse
                let error_msg = match response.json::<ApiResponse<()>>().await {
                    Ok(data) => data.message,
                    Err(_) => response.text().await.unwrap_or_else(|_| format!("HTTP {}", response.status())),
                };
                Err(ApiError { message: error_msg })
            }
        },
        Err(err) => Err(ApiError { message: format!("Network error: {}", err) }),
    }
}

// Callback versions for Yew components
pub fn get_projects(callback: Callback<Result<Vec<Project>, ApiError>>) {
    spawn_local(async move {
        let result = fetch_projects(1, 1000, None, None).await;
        match result {
            Ok(paginated) => callback.emit(Ok(paginated.items)),
            Err(e) => callback.emit(Err(e)),
        }
    });
}
