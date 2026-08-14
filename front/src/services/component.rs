use urlencoding;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use crate::services::auth;
use crate::types::ApiResponse;
use common::models::{Component, ComponentStatus, PaginatedResult};

const API_BASE_URL: &str = "/api/v1/components";

async fn request_get(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    auth::get(url).await
}

async fn request_put<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    auth::put_json(url, body).await
}

async fn request_post<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    auth::post_json(url, body).await
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

pub async fn fetch_components(
    page: Option<usize>,
    page_size: Option<usize>,
    client_id: Option<String>,
    component_type: Option<String>,
    status: Option<String>,
    q: Option<String>,
) -> Result<PaginatedResult<Component>, ApiError> {
    let mut url = API_BASE_URL.to_string();
    let mut params = Vec::new();

    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(ps) = page_size {
        params.push(format!("page_size={}", ps));
    }

    if let Some(cid) = client_id {
        if !cid.is_empty() {
            params.push(format!("client_id={}", urlencoding::encode(&cid)));
        }
    }
    if let Some(ctype) = component_type {
        if !ctype.is_empty() && ctype != "all" {
            params.push(format!("component_type={}", urlencoding::encode(&ctype)));
        }
    }
    if let Some(stat) = status {
        if !stat.is_empty() && stat != "all" {
            params.push(format!("status={}", urlencoding::encode(&stat)));
        }
    }
    if let Some(query) = q {
        if !query.is_empty() {
            params.push(format!("q={}", urlencoding::encode(&query)));
        }
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response
                    .json::<ApiResponse<PaginatedResult<Component>>>()
                    .await
                {
                    Ok(data) => Ok(data.data.unwrap_or_else(|| PaginatedResult {
                        items: vec![],
                        total: 0,
                        page: 1,
                        page_size: 10,
                        total_pages: 0,
                    })),
                    Err(err) => Err(ApiError {
                        message: format!("Failed to parse components: {}", err),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Failed to fetch components: HTTP {}", response.status()),
                })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

#[allow(dead_code)]
pub async fn get_component(id: &str) -> Result<Component, ApiError> {
    let url = format!("{}/{}", API_BASE_URL, id);
    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Component>>().await {
                    Ok(data) => Ok(data.data.ok_or(ApiError {
                        message: "No data returned".to_string(),
                    })?),
                    Err(err) => Err(ApiError {
                        message: format!("Failed to parse component: {}", err),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Failed to get component: HTTP {}", response.status()),
                })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

pub async fn create_component(component: &Component) -> Result<Component, ApiError> {
    match request_post(API_BASE_URL, component).await {
        Ok(response) => {
            if response.status() == 201 {
                match response.json::<ApiResponse<Component>>().await {
                    Ok(data) => Ok(data.data.ok_or(ApiError {
                        message: "No data returned".to_string(),
                    })?),
                    Err(err) => Err(ApiError {
                        message: format!("Failed to parse created component: {}", err),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Failed to create component: HTTP {}", response.status()),
                })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

pub async fn update_component(id: &str, component: &Component) -> Result<Component, ApiError> {
    let url = format!("{}/{}", API_BASE_URL, id);
    match request_put(&url, component).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Component>>().await {
                    Ok(data) => Ok(data.data.ok_or(ApiError {
                        message: "No data returned".to_string(),
                    })?),
                    Err(err) => Err(ApiError {
                        message: format!("Failed to parse updated component: {}", err),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Failed to update component: HTTP {}", response.status()),
                })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

pub async fn batch_update_components(
    ids: Vec<String>,
    status: Option<ComponentStatus>,
) -> Result<usize, ApiError> {
    let url = format!("{}/batch/update", API_BASE_URL);
    let body = serde_json::json!({ "ids": ids, "status": status });
    match request_post(&url, &body).await {
        Ok(response) => {
            if response.status() == 200 || response.status() == 207 {
                match response.json::<ApiResponse<usize>>().await {
                    Ok(data) => Ok(data.data.unwrap_or(0)),
                    Err(err) => Err(ApiError {
                        message: format!("Failed to parse response: {}", err),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Failed to batch update: HTTP {}", response.status()),
                })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

pub async fn batch_create_components(components: Vec<Component>) -> Result<usize, ApiError> {
    let url = format!("{}/batch/create", API_BASE_URL);
    let body = serde_json::json!({ "components": components });
    match request_post(&url, &body).await {
        Ok(response) => {
            if response.status() == 200 || response.status() == 207 {
                match response.json::<ApiResponse<usize>>().await {
                    Ok(data) => Ok(data.data.unwrap_or(0)),
                    Err(err) => Err(ApiError {
                        message: format!("Failed to parse response: {}", err),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Failed to batch create: HTTP {}", response.status()),
                })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

#[allow(dead_code)]
pub async fn batch_delete_components(ids: Vec<String>) -> Result<usize, ApiError> {
    let url = format!("{}/batch/delete", API_BASE_URL);
    let body = serde_json::json!({ "ids": ids });
    match request_post(&url, &body).await {
        Ok(response) => {
            if response.status() == 200 || response.status() == 207 {
                match response.json::<ApiResponse<usize>>().await {
                    Ok(data) => Ok(data.data.unwrap_or(0)),
                    Err(err) => Err(ApiError {
                        message: format!("Failed to parse response: {}", err),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Failed to batch delete: HTTP {}", response.status()),
                })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

// Callback versions for Yew components
pub fn get_components(
    page: Option<usize>,
    page_size: Option<usize>,
    client_id: Option<String>,
    component_type: Option<String>,
    status: Option<String>,
    q: Option<String>,
    callback: Callback<Result<PaginatedResult<Component>, ApiError>>,
) {
    spawn_local(async move {
        let result = fetch_components(page, page_size, client_id, component_type, status, q).await;
        callback.emit(result);
    });
}
