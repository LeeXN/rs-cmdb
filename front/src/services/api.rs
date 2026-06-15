use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use log::{error, info};
use urlencoding;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::Callback;

use crate::stores::auth_store::AuthStore;
use crate::types::{
    ApiResponse, ChangePasswordRequest, Client, CreateUserRequest, DetailedStats, Dictionary,
    FilterCriteria, FilterOptions, Hardware, HardwareHistoryEntry, LoginRequest, LoginResponse,
    PaginatedResult, Person, Project, Rack, UpdateUserRequest, User,
};

const API_BASE_URL: &str = "/api/v1";

fn get_auth_header() -> Option<String> {
    // Try to get AuthStore from LocalStorage. Yewdux usually uses the struct name.
    // We try "auth_store" (snake_case) first as it is common convention, then "AuthStore".
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

    // Debug logging if token not found
    info!("No auth token found in LocalStorage (checked 'auth_store' and 'AuthStore')");
    None
}

async fn request_get(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::get(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.send().await?;

    if response.status() == 401 {
        info!("Received 401 Unauthorized, redirecting to login (v2)...");
        // Clear auth token
        LocalStorage::delete("auth_store");
        LocalStorage::delete("AuthStore");

        // Redirect to login
        if let Some(win) = window() {
            let _ = win.location().set_href("/login");
        }
    }

    Ok(response)
}

async fn request_post<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::post(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.json(body)?.send().await?;

    // Don't redirect for login endpoint itself, as 401 there means invalid credentials
    if response.status() == 401 && !url.ends_with("/auth/login") {
        info!("Received 401 Unauthorized, redirecting to login (v2)...");
        // Clear auth token
        LocalStorage::delete("auth_store");
        LocalStorage::delete("AuthStore");

        // Redirect to login
        if let Some(win) = window() {
            let _ = win.location().set_href("/login");
        }
    }

    Ok(response)
}

async fn request_put<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::put(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.json(body)?.send().await?;

    if response.status() == 401 {
        info!("Received 401 Unauthorized, redirecting to login (v2)...");
        LocalStorage::delete("auth_store");
        LocalStorage::delete("AuthStore");

        if let Some(win) = window() {
            let _ = win.location().set_href("/login");
        }
    }

    Ok(response)
}

async fn request_delete(url: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let mut req = Request::delete(url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    let response = req.send().await?;

    if response.status() == 401 {
        info!("Received 401 Unauthorized, redirecting to login (v2)...");
        LocalStorage::delete("auth_store");
        LocalStorage::delete("AuthStore");

        if let Some(win) = window() {
            let _ = win.location().set_href("/login");
        }
    }

    Ok(response)
}

/// 通用 API 请求错误
#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

// const API_BASE_URL: &str = "/api/v1"; // Removed duplicate definition
// ...existing code...
// FilterOptions类型现在从common::models导入，所以移除重复定义

/// 异步获取客户端列表
pub async fn fetch_clients(
    page: usize,
    page_size: usize,
    search: Option<String>,
    os: Option<String>,
    status: Option<String>,
) -> Result<PaginatedResult<Client>, ApiError> {
    let mut url = format!(
        "{}/clients?page={}&page_size={}",
        API_BASE_URL, page, page_size
    );

    if let Some(s) = search {
        if !s.is_empty() {
            url.push_str(&format!("&search={}", urlencoding::encode(&s)));
        }
    }

    if let Some(o) = os {
        if !o.is_empty() && o != "all" {
            url.push_str(&format!("&os={}", urlencoding::encode(&o)));
        }
    }

    if let Some(st) = status {
        if !st.is_empty() && st != "all" {
            url.push_str(&format!("&status={}", urlencoding::encode(&st)));
        }
    }

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response
                    .json::<ApiResponse<PaginatedResult<Client>>>()
                    .await
                {
                    Ok(data) => {
                        if let Some(result) = data.data {
                            Ok(result)
                        } else {
                            Err(ApiError {
                                message: "无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析客户端数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求客户端列表失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步搜索客户端 (Deprecated: use fetch_clients instead)
#[allow(dead_code)]
pub async fn search_clients(
    search_term: Option<String>,
    os_filter: Option<String>,
    status_filter: Option<String>,
) -> Result<Vec<Client>, ApiError> {
    // For backward compatibility, fetch first page with large size
    match fetch_clients(1, 1000, search_term, os_filter, status_filter).await {
        Ok(result) => Ok(result.items),
        Err(e) => Err(e),
    }
}

/// 异步获取客户端列表 (Deprecated: use fetch_clients with pagination instead)
// pub async fn fetch_clients() -> Result<Vec<Client>, ApiError> { ... } - Removed
/// 异步获取客户端详情
pub async fn fetch_client(client_id: &str) -> Result<Client, ApiError> {
    let url = format!("{}/clients/{}", API_BASE_URL, client_id);

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Client>>().await {
                    Ok(data) => {
                        if let Some(client) = data.data {
                            Ok(client)
                        } else {
                            Err(ApiError {
                                message: "客户端不存在".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析客户端数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求客户端详情失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("请求客户端详情失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 获取硬件信息
pub async fn fetch_hardware_info(client_id: &str) -> Result<Hardware, ApiError> {
    let url = format!("{}/clients/{}/hardware", API_BASE_URL, client_id);

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Hardware>>().await {
                    Ok(data) => {
                        if let Some(hardware_info) = data.data {
                            Ok(hardware_info)
                        } else {
                            Err(ApiError {
                                message: "硬件信息不存在".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析硬件信息失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求硬件信息失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("请求硬件信息失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 获取硬件历史信息
pub async fn fetch_hardware_history(
    client_id: &str,
) -> Result<Vec<HardwareHistoryEntry>, ApiError> {
    let url = format!("{}/clients/{}/hardware/history", API_BASE_URL, client_id);

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response
                    .json::<ApiResponse<Vec<HardwareHistoryEntry>>>()
                    .await
                {
                    Ok(data) => {
                        if let Some(history) = data.data {
                            Ok(history)
                        } else {
                            Ok(Vec::new()) // 返回空历史记录
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析硬件历史失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求硬件历史失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("请求硬件历史失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 获取客户端列表
pub fn get_clients(
    page: usize,
    page_size: usize,
    search: Option<String>,
    os: Option<String>,
    status: Option<String>,
    callback: Callback<Result<PaginatedResult<Client>, ApiError>>,
) {
    spawn_local(async move {
        let result = fetch_clients(page, page_size, search, os, status).await;
        callback.emit(result);
    });
}

/// 搜索客户端
#[allow(dead_code)]
pub fn search_clients_sync(
    search_term: Option<String>,
    os_filter: Option<String>,
    status_filter: Option<String>,
    callback: Callback<Result<Vec<Client>, ApiError>>,
) {
    spawn_local(async move {
        let result = search_clients(search_term, os_filter, status_filter).await;
        callback.emit(result);
    });
}

/// 使用回调方式获取客户端详情
pub fn get_client(client_id: String, callback: Callback<Result<Client, ApiError>>) {
    spawn_local(async move {
        let result = fetch_client(&client_id).await;
        callback.emit(result);
    });
}

/// 异步删除客户端
pub async fn delete_client(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/clients/{}", API_BASE_URL, id);

    match request_delete(&url).await {
        Ok(response) => {
            if response.status() == 200 || response.status() == 204 {
                Ok(())
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("删除客户端失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 使用回调方式获取客户端硬件信息
pub fn get_hardware_info(client_id: String, callback: Callback<Result<Hardware, ApiError>>) {
    spawn_local(async move {
        let result = fetch_hardware_info(&client_id).await;
        callback.emit(result);
    });
}

/// 异步获取详细统计信息
pub async fn fetch_detailed_stats() -> Result<DetailedStats, ApiError> {
    let url = format!("{}/stats/detailed", API_BASE_URL);

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<DetailedStats>>().await {
                    Ok(data) => Ok(data.data.unwrap_or_default()),
                    Err(err) => {
                        let error_msg = format!("解析详细统计数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求详细统计数据失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步根据筛选条件获取客户端列表
#[allow(dead_code)]
pub async fn fetch_clients_by_filter(filter: FilterCriteria) -> Result<Vec<Client>, ApiError> {
    let url = format!("{}/clients/filter", API_BASE_URL);

    match request_post(&url, &filter).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Vec<Client>>>().await {
                    Ok(data) => Ok(data.data.unwrap_or_default()),
                    Err(err) => {
                        let error_msg = format!("解析筛选结果失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("筛选客户端失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 获取详细统计信息
pub fn get_detailed_stats(callback: Callback<Result<DetailedStats, ApiError>>) {
    spawn_local(async move {
        let result = fetch_detailed_stats().await;
        callback.emit(result);
    });
}

/// 根据筛选条件获取客户端列表
#[allow(dead_code)]
pub fn get_clients_by_filter(
    filter: FilterCriteria,
    callback: Callback<Result<Vec<Client>, ApiError>>,
) {
    spawn_local(async move {
        let result = fetch_clients_by_filter(filter).await;
        callback.emit(result);
    });
}

/// 异步获取导出数据
pub async fn fetch_export_data() -> Result<Vec<crate::types::ClientHardwareExport>, ApiError> {
    let url = format!("{}/stats/export", API_BASE_URL);

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response
                    .json::<ApiResponse<Vec<crate::types::ClientHardwareExport>>>()
                    .await
                {
                    Ok(data) => Ok(data.data.unwrap_or_default()),
                    Err(err) => {
                        let error_msg = format!("解析导出数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求导出数据失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("请求导出数据失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 同步获取导出数据
pub fn get_export_data(
    callback: Callback<Result<Vec<crate::types::ClientHardwareExport>, ApiError>>,
) {
    spawn_local(async move {
        let result = fetch_export_data().await;
        callback.emit(result);
    });
}

/// 异步获取筛选选项 - 从后端API实时获取
pub async fn fetch_filter_options() -> Result<FilterOptions, ApiError> {
    let url = format!("{}/filter_options", API_BASE_URL);

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<FilterOptions>>().await {
                    Ok(data) => {
                        let filter_options = data.data.unwrap_or_default();
                        Ok(filter_options)
                    }
                    Err(err) => {
                        let error_msg = format!("解析筛选选项失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求筛选选项失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 获取筛选选项
pub fn get_filter_options(callback: Callback<Result<FilterOptions, ApiError>>) {
    spawn_local(async move {
        let result = fetch_filter_options().await;
        callback.emit(result);
    });
}

/// Fetch clients filtered by hardware specifications
#[allow(clippy::too_many_arguments)]
pub async fn fetch_clients_by_hardware_filter(
    search_term: Option<String>,
    os_filter: Option<String>,
    os_kernel_filter: Option<String>,
    cpu_vendor_filter: Option<String>,
    cpu_model_filter: Option<String>,
    gpu_vendor_filter: Option<String>,
    gpu_model_filter: Option<String>,
    memory_min_filter: Option<u32>,
    memory_max_filter: Option<u32>,
    server_vendor_filter: Option<String>,
    server_model_filter: Option<String>,
    network_type_filter: Option<String>,
    network_model_filter: Option<String>,
    storage_type_filter: Option<String>,
    status_filter: Option<String>,
    client_status_filter: Option<String>,
    environment_filter: Option<String>,
    rack_id_filter: Option<String>,
    project_id_filter: Option<String>,
    owner_id_filter: Option<String>,
) -> Result<Vec<Client>, ApiError> {
    let mut url = format!("{}/clients/filter_hardware", API_BASE_URL);
    let mut params = Vec::new();

    if let Some(term) = search_term {
        if !term.is_empty() {
            params.push(format!("search_term={}", urlencoding::encode(&term)));
        }
    }

    if let Some(os) = os_filter {
        if !os.is_empty() && os != "全部" {
            params.push(format!("os_filter={}", urlencoding::encode(&os)));
        }
    }

    if let Some(kernel) = os_kernel_filter {
        if !kernel.is_empty() && kernel != "全部" {
            params.push(format!("os_kernel_filter={}", urlencoding::encode(&kernel)));
        }
    }

    if let Some(cpu_vendor) = cpu_vendor_filter {
        if !cpu_vendor.is_empty() && cpu_vendor != "全部" {
            params.push(format!(
                "cpu_vendor_filter={}",
                urlencoding::encode(&cpu_vendor)
            ));
        }
    }

    if let Some(cpu_model) = cpu_model_filter {
        if !cpu_model.is_empty() && cpu_model != "全部" {
            params.push(format!(
                "cpu_model_filter={}",
                urlencoding::encode(&cpu_model)
            ));
        }
    }

    if let Some(gpu_vendor) = gpu_vendor_filter {
        if !gpu_vendor.is_empty() && gpu_vendor != "全部" {
            params.push(format!(
                "gpu_vendor_filter={}",
                urlencoding::encode(&gpu_vendor)
            ));
        }
    }

    if let Some(gpu_model) = gpu_model_filter {
        if !gpu_model.is_empty() && gpu_model != "全部" {
            params.push(format!(
                "gpu_model_filter={}",
                urlencoding::encode(&gpu_model)
            ));
        }
    }

    if let Some(min_mem) = memory_min_filter {
        params.push(format!("memory_min_filter={}", min_mem));
    }

    if let Some(max_mem) = memory_max_filter {
        params.push(format!("memory_max_filter={}", max_mem));
    }

    if let Some(server_vendor) = server_vendor_filter {
        if !server_vendor.is_empty() && server_vendor != "全部" {
            params.push(format!(
                "server_vendor_filter={}",
                urlencoding::encode(&server_vendor)
            ));
        }
    }

    if let Some(server_model) = server_model_filter {
        if !server_model.is_empty() && server_model != "全部" {
            params.push(format!(
                "server_model_filter={}",
                urlencoding::encode(&server_model)
            ));
        }
    }

    if let Some(network_type) = network_type_filter {
        if !network_type.is_empty() && network_type != "全部" {
            params.push(format!(
                "network_type_filter={}",
                urlencoding::encode(&network_type)
            ));
        }
    }

    if let Some(network_model) = network_model_filter {
        if !network_model.is_empty() && network_model != "全部" {
            params.push(format!(
                "network_model_filter={}",
                urlencoding::encode(&network_model)
            ));
        }
    }

    if let Some(storage_type) = storage_type_filter {
        if !storage_type.is_empty() && storage_type != "全部" {
            params.push(format!(
                "storage_type_filter={}",
                urlencoding::encode(&storage_type)
            ));
        }
    }

    if let Some(status) = status_filter {
        if !status.is_empty() && status != "全部" {
            params.push(format!("status_filter={}", urlencoding::encode(&status)));
        }
    }

    if let Some(client_status) = client_status_filter {
        if !client_status.is_empty() && client_status != "全部" {
            params.push(format!(
                "client_status_filter={}",
                urlencoding::encode(&client_status)
            ));
        }
    }

    if let Some(env) = environment_filter {
        if !env.is_empty() && env != "全部" {
            params.push(format!("environment_filter={}", urlencoding::encode(&env)));
        }
    }

    if let Some(rack) = rack_id_filter {
        if !rack.is_empty() && rack != "全部" {
            params.push(format!("rack_id_filter={}", urlencoding::encode(&rack)));
        }
    }

    if let Some(project) = project_id_filter {
        if !project.is_empty() && project != "全部" {
            params.push(format!(
                "project_id_filter={}",
                urlencoding::encode(&project)
            ));
        }
    }

    if let Some(owner) = owner_id_filter {
        if !owner.is_empty() && owner != "全部" {
            params.push(format!("owner_id_filter={}", urlencoding::encode(&owner)));
        }
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Vec<Client>>>().await {
                    Ok(data) => Ok(data.data.unwrap_or_default()),
                    Err(err) => {
                        let error_msg = format!("解析硬件筛选结果失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("硬件筛选失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("硬件筛选网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// Callback-based hardware filter function
#[allow(clippy::too_many_arguments)]
pub fn get_clients_by_hardware_filter(
    search_term: Option<String>,
    os_filter: Option<String>,
    os_kernel_filter: Option<String>,
    cpu_vendor_filter: Option<String>,
    cpu_model_filter: Option<String>,
    gpu_vendor_filter: Option<String>,
    gpu_model_filter: Option<String>,
    memory_min_filter: Option<u32>,
    memory_max_filter: Option<u32>,
    server_vendor_filter: Option<String>,
    server_model_filter: Option<String>,
    network_type_filter: Option<String>,
    network_model_filter: Option<String>,
    storage_type_filter: Option<String>,
    status_filter: Option<String>,
    client_status_filter: Option<String>,
    environment_filter: Option<String>,
    rack_id_filter: Option<String>,
    project_id_filter: Option<String>,
    owner_id_filter: Option<String>,
    callback: Callback<Result<Vec<Client>, ApiError>>,
) {
    spawn_local(async move {
        let result = fetch_clients_by_hardware_filter(
            search_term,
            os_filter,
            os_kernel_filter,
            cpu_vendor_filter,
            cpu_model_filter,
            gpu_vendor_filter,
            gpu_model_filter,
            memory_min_filter,
            memory_max_filter,
            server_vendor_filter,
            server_model_filter,
            network_type_filter,
            network_model_filter,
            storage_type_filter,
            status_filter,
            client_status_filter,
            environment_filter,
            rack_id_filter,
            project_id_filter,
            owner_id_filter,
        )
        .await;
        callback.emit(result);
    });
}

/// 异步获取基于设备ID列表的硬件筛选选项
#[allow(dead_code)]
pub async fn fetch_filter_options_by_client_ids(
    client_ids: Vec<String>,
) -> Result<FilterOptions, ApiError> {
    let ids_param = client_ids.join(",");
    let url = format!(
        "/api/v1/filter_options_by_ids?client_ids={}",
        urlencoding::encode(&ids_param)
    );

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<FilterOptions>>().await {
                    Ok(data) => Ok(data.data.unwrap_or_default()),
                    Err(err) => {
                        let error_msg = format!("解析硬件筛选选项失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("获取硬件筛选选项失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("获取硬件筛选选项网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 根据设备ID列表获取硬件筛选选项
#[allow(dead_code)]
pub fn get_filter_options_by_client_ids(
    client_ids: Vec<String>,
    callback: Callback<Result<FilterOptions, ApiError>>,
) {
    spawn_local(async move {
        let result = fetch_filter_options_by_client_ids(client_ids).await;
        callback.emit(result);
    });
}

/// 登录
pub async fn login(request: LoginRequest) -> Result<LoginResponse, ApiError> {
    let url = format!("{}/auth/login", API_BASE_URL);

    match request_post(&url, &request).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<LoginResponse>>().await {
                    Ok(data) => {
                        if let Some(login_response) = data.data {
                            Ok(login_response)
                        } else {
                            Err(ApiError {
                                message: "登录失败: 无数据".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析登录响应失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                let error_msg = format!("登录失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步更新客户端信息
pub async fn update_client(client_id: &str, client: &Client) -> Result<Client, ApiError> {
    let url = format!("{}/clients/{}", API_BASE_URL, client_id);

    match request_put(&url, client).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Client>>().await {
                    Ok(data) => {
                        if let Some(updated_client) = data.data {
                            Ok(updated_client)
                        } else {
                            Err(ApiError {
                                message: "更新失败: 无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析更新响应失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                // Try to parse error message from response
                let error_msg = match response.json::<ApiResponse<Client>>().await {
                    Ok(data) => data.message,
                    Err(_) => format!("更新客户端失败: HTTP {}", response.status()),
                };
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步获取人员列表
pub async fn fetch_persons(
    page: usize,
    page_size: usize,
    search: Option<String>,
    department: Option<String>,
) -> Result<PaginatedResult<Person>, ApiError> {
    let mut url = format!(
        "{}/users?page={}&page_size={}",
        API_BASE_URL, page, page_size
    );

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
                match response
                    .json::<ApiResponse<PaginatedResult<Person>>>()
                    .await
                {
                    Ok(data) => {
                        if let Some(result) = data.data {
                            Ok(result)
                        } else {
                            Err(ApiError {
                                message: "无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析人员数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求人员列表失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步获取项目列表
pub async fn fetch_projects(
    page: usize,
    page_size: usize,
    search: Option<String>,
    department: Option<String>,
) -> Result<PaginatedResult<Project>, ApiError> {
    let mut url = format!(
        "{}/projects?page={}&page_size={}",
        API_BASE_URL, page, page_size
    );

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
                match response
                    .json::<ApiResponse<PaginatedResult<Project>>>()
                    .await
                {
                    Ok(data) => {
                        if let Some(result) = data.data {
                            Ok(result)
                        } else {
                            Err(ApiError {
                                message: "无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析项目数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求项目列表失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步获取字典列表
pub async fn fetch_dictionaries(category: Option<String>) -> Result<Vec<Dictionary>, ApiError> {
    let mut url = format!("{}/dictionaries", API_BASE_URL);
    if let Some(cat) = category {
        url.push_str(&format!("?category={}", urlencoding::encode(&cat)));
    }

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Vec<Dictionary>>>().await {
                    Ok(data) => Ok(data.data.unwrap_or_default()),
                    Err(err) => {
                        let error_msg = format!("解析字典数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求字典列表失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步创建字典项
pub async fn create_dictionary(item: &Dictionary) -> Result<Dictionary, ApiError> {
    let url = format!("{}/dictionaries", API_BASE_URL);

    match request_post(&url, item).await {
        Ok(response) => {
            if response.status() == 201 {
                match response.json::<ApiResponse<Dictionary>>().await {
                    Ok(data) => {
                        if let Some(new_item) = data.data {
                            Ok(new_item)
                        } else {
                            Err(ApiError {
                                message: "创建失败: 无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析创建响应失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("创建字典项失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步更新字典项
pub async fn update_dictionary(id: &str, item: &Dictionary) -> Result<Dictionary, ApiError> {
    let url = format!("{}/dictionaries/{}", API_BASE_URL, id);

    match request_put(&url, item).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Dictionary>>().await {
                    Ok(data) => {
                        if let Some(updated_item) = data.data {
                            Ok(updated_item)
                        } else {
                            Err(ApiError {
                                message: "更新失败: 无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析更新响应失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("更新字典项失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步删除字典项
pub async fn delete_dictionary(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/dictionaries/{}", API_BASE_URL, id);

    let mut req = Request::delete(&url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }

    match req.send().await {
        Ok(response) => {
            if response.status() == 200 {
                Ok(())
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                // Try to parse error message from ApiResponse
                let error_msg = match response.json::<ApiResponse<()>>().await {
                    Ok(data) => data.message,
                    Err(_) => response
                        .text()
                        .await
                        .unwrap_or_else(|_| format!("HTTP {}", response.status())),
                };
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步获取机柜列表
pub async fn fetch_racks(
    page: usize,
    page_size: usize,
    search: Option<String>,
    location: Option<String>,
) -> Result<PaginatedResult<Rack>, ApiError> {
    let mut url = format!(
        "{}/racks?page={}&page_size={}",
        API_BASE_URL, page, page_size
    );

    if let Some(s) = search {
        if !s.is_empty() {
            url.push_str(&format!("&search={}", urlencoding::encode(&s)));
        }
    }

    if let Some(l) = location {
        if !l.is_empty() {
            url.push_str(&format!("&location={}", urlencoding::encode(&l)));
        }
    }

    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<PaginatedResult<Rack>>>().await {
                    Ok(data) => {
                        if let Some(result) = data.data {
                            Ok(result)
                        } else {
                            Err(ApiError {
                                message: "无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析机柜数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("请求机柜列表失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步创建机柜
pub async fn create_rack(rack: &Rack) -> Result<Rack, ApiError> {
    let url = format!("{}/racks", API_BASE_URL);

    match request_post(&url, rack).await {
        Ok(response) => {
            if response.status() == 201 {
                match response.json::<ApiResponse<Rack>>().await {
                    Ok(data) => {
                        if let Some(new_rack) = data.data {
                            Ok(new_rack)
                        } else {
                            Err(ApiError {
                                message: "创建失败: 无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析创建响应失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("创建机柜失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步更新机柜
pub async fn update_rack(id: &str, rack: &Rack) -> Result<Rack, ApiError> {
    let url = format!("{}/racks/{}", API_BASE_URL, id);

    match request_put(&url, rack).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Rack>>().await {
                    Ok(data) => {
                        if let Some(updated_rack) = data.data {
                            Ok(updated_rack)
                        } else {
                            Err(ApiError {
                                message: "更新失败: 无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析更新响应失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("更新机柜失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// 异步删除机柜
pub async fn delete_rack(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/racks/{}", API_BASE_URL, id);

    let mut req = Request::delete(&url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }

    match req.send().await {
        Ok(response) => {
            if response.status() == 200 {
                Ok(())
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                // Try to parse error message from ApiResponse
                let error_msg = match response.json::<ApiResponse<()>>().await {
                    Ok(data) => data.message,
                    Err(_) => response
                        .text()
                        .await
                        .unwrap_or_else(|_| format!("HTTP {}", response.status())),
                };
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("网络请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// Change password
pub async fn change_password(request: ChangePasswordRequest) -> Result<(), ApiError> {
    let url = format!("{}/auth/change-password", API_BASE_URL);
    match request_post(&url, &request).await {
        Ok(response) => {
            if response.status() == 200 {
                Ok(())
            } else {
                let error_msg = format!("Change password failed: HTTP {}", response.status());
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => Err(ApiError {
            message: format!("Network error: {}", err),
        }),
    }
}

/// Fetch users
pub async fn fetch_users() -> Result<Vec<User>, ApiError> {
    let url = format!("{}/accounts", API_BASE_URL);
    match request_get(&url).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<Vec<User>>>().await {
                    Ok(data) => Ok(data.data.unwrap_or_default()),
                    Err(e) => Err(ApiError {
                        message: format!("Parse error: {}", e),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Fetch users failed: HTTP {}", response.status()),
                })
            }
        }
        Err(e) => Err(ApiError {
            message: format!("Network error: {}", e),
        }),
    }
}

/// Create user
pub async fn create_user(request: CreateUserRequest) -> Result<User, ApiError> {
    let url = format!("{}/auth/register", API_BASE_URL);
    match request_post(&url, &request).await {
        Ok(response) => {
            if response.status() == 201 {
                match response.json::<ApiResponse<User>>().await {
                    Ok(data) => Ok(data.data.unwrap()),
                    Err(e) => Err(ApiError {
                        message: format!("Parse error: {}", e),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Create user failed: HTTP {}", response.status()),
                })
            }
        }
        Err(e) => Err(ApiError {
            message: format!("Network error: {}", e),
        }),
    }
}

/// Update user
pub async fn update_user(id: &str, request: UpdateUserRequest) -> Result<User, ApiError> {
    let url = format!("{}/accounts/{}", API_BASE_URL, id);
    match request_put(&url, &request).await {
        Ok(response) => {
            if response.status() == 200 {
                match response.json::<ApiResponse<User>>().await {
                    Ok(data) => Ok(data.data.unwrap()),
                    Err(e) => Err(ApiError {
                        message: format!("Parse error: {}", e),
                    }),
                }
            } else {
                Err(ApiError {
                    message: format!("Update user failed: HTTP {}", response.status()),
                })
            }
        }
        Err(e) => Err(ApiError {
            message: format!("Network error: {}", e),
        }),
    }
}

/// Delete user
pub async fn delete_user(id: &str) -> Result<(), ApiError> {
    let url = format!("{}/accounts/{}", API_BASE_URL, id);
    let mut req = Request::delete(&url);
    if let Some(auth) = get_auth_header() {
        req = req.header("Authorization", &auth);
    }
    match req.send().await {
        Ok(response) => {
            if response.status() == 200 {
                Ok(())
            } else {
                Err(ApiError {
                    message: format!("Delete user failed: HTTP {}", response.status()),
                })
            }
        }
        Err(e) => Err(ApiError {
            message: format!("Network error: {}", e),
        }),
    }
}

/// Export filtered clients with hardware data
pub async fn export_filtered_clients(
    request: crate::types::ExportFilterRequest,
) -> Result<crate::types::ExportFilterResponse, ApiError> {
    let url = format!("{}/clients/export_filtered", API_BASE_URL);

    match request_post(&url, &request).await {
        Ok(response) => {
            if response.status() == 200 {
                match response
                    .json::<ApiResponse<crate::types::ExportFilterResponse>>()
                    .await
                {
                    Ok(data) => {
                        if let Some(result) = data.data {
                            Ok(result)
                        } else {
                            Err(ApiError {
                                message: "导出失败: 无数据返回".to_string(),
                            })
                        }
                    }
                    Err(err) => {
                        let error_msg = format!("解析导出数据失败: {}", err);
                        error!("{}", error_msg);
                        Err(ApiError { message: error_msg })
                    }
                }
            } else if response.status() == 400 {
                match response
                    .json::<ApiResponse<crate::types::ExportFilterResponse>>()
                    .await
                {
                    Ok(data) => Err(ApiError {
                        message: data.message,
                    }),
                    Err(_) => Err(ApiError {
                        message: "导出失败: 请求参数错误或导出数量超过限制".to_string(),
                    }),
                }
            } else {
                if response.status() == 401 {
                    return Err(ApiError {
                        message: "Unauthorized".to_string(),
                    });
                }
                let error_msg = format!("导出失败: HTTP {}", response.status());
                error!("{}", error_msg);
                Err(ApiError { message: error_msg })
            }
        }
        Err(err) => {
            let error_msg = format!("导出请求失败: {}", err);
            error!("{}", error_msg);
            Err(ApiError { message: error_msg })
        }
    }
}

/// Callback-based export filtered clients
pub fn get_export_filtered_clients(
    request: crate::types::ExportFilterRequest,
    callback: Callback<Result<crate::types::ExportFilterResponse, ApiError>>,
) {
    spawn_local(async move {
        let result = export_filtered_clients(request).await;
        callback.emit(result);
    });
}
