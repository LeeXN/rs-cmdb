use crate::queue::{Message, MessageQueue};
use crate::repository::{
    client_repository::ClientRepository, component_repository::ComponentRepository,
    hardware_repository::HardwareRepository,
};
use crate::service::{client_service::ClientService, validation_service::ValidationService};
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use common::models::{ApiResponse, Client, ClientQuery, PaginatedResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub os: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HardwareFilterQuery {
    pub search_term: Option<String>,
    pub os_filter: Option<String>,
    pub os_kernel_filter: Option<String>,
    pub cpu_vendor_filter: Option<String>,
    pub cpu_model_filter: Option<String>,
    pub gpu_vendor_filter: Option<String>,
    pub gpu_model_filter: Option<String>,
    pub memory_min_filter: Option<u32>,
    pub memory_max_filter: Option<u32>,
    pub server_vendor_filter: Option<String>,
    pub server_model_filter: Option<String>,
    pub network_type_filter: Option<String>,
    pub network_model_filter: Option<String>,
    pub storage_type_filter: Option<String>,
    pub status_filter: Option<String>,
    pub client_status_filter: Option<String>,
    pub environment_filter: Option<String>,
    pub rack_id_filter: Option<String>,
    pub project_id_filter: Option<String>,
    pub owner_id_filter: Option<String>,
}

/// List all clients
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn list_clients(
    Query(query): Query<ClientQuery>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    match client_repo.list_all().await {
        Ok(mut clients) => {
            // Filter by search term
            if let Some(ref search) = query.search {
                let search_lower = search.to_lowercase();
                clients.retain(|c| {
                    c.hostname.to_lowercase().contains(&search_lower)
                        || c.ip_address.contains(&search_lower)
                        || c.os
                            .as_ref()
                            .is_some_and(|os| os.to_lowercase().contains(&search_lower))
                });
            }

            // Filter by OS
            if let Some(ref os) = query.os
                && os != "all"
            {
                clients.retain(|c| c.os.as_ref() == Some(os));
            }

            // Filter by status
            if let Some(ref status) = query.status
                && status != "all"
            {
                let now = chrono::Utc::now();
                clients.retain(|c| {
                    let is_online = c
                        .last_seen
                        .as_ref()
                        .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                        .map(|dt| {
                            let duration =
                                now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                            duration.num_minutes() <= 5
                        })
                        .unwrap_or(false);

                    match status.as_str() {
                        "online" => is_online,
                        "offline" => !is_online,
                        _ => true,
                    }
                });
            }

            // Sort by hostname
            clients.sort_by(|a, b| a.hostname.cmp(&b.hostname));

            // Pagination
            let total = clients.len();
            let page = query.page.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            let total_pages = (total as f64 / page_size as f64).ceil() as usize;

            let start = (page - 1) * page_size;
            let end = std::cmp::min(start + page_size, total);

            let items = if start < total {
                clients[start..end].to_vec()
            } else {
                Vec::new()
            };

            info!(
                "Listed {} clients (page {}/{})",
                items.len(),
                page,
                total_pages
            );

            let result = PaginatedResult {
                items,
                total,
                page,
                page_size,
                total_pages,
            };

            let response = ApiResponse {
                status: 200,
                message: "Clients retrieved successfully".to_string(),
                data: Some(result),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to list clients: {}", err);
            let response = ApiResponse::<PaginatedResult<Client>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Get a client by ID
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn get_client(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    match client_repo.get(&client_id).await {
        Ok(Some(client)) => {
            let response = ApiResponse {
                status: 200,
                message: "Client retrieved successfully".to_string(),
                data: Some(client),
            };

            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = ApiResponse::<Client> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };

            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to get client {}: {}", client_id, err);
            let response = ApiResponse::<Client> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Register a new client
#[debug_handler]
#[instrument(skip(client_service, message_queue, validation_service, registration))]
pub async fn register_client(
    Extension(client_service): Extension<Arc<ClientService>>,
    Extension(message_queue): Extension<Arc<dyn MessageQueue>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Json(registration): Json<Client>,
) -> impl IntoResponse {
    info!("Registering client: {}", registration.hostname);

    // Validate references
    if let Some(rack_id) = &registration.rack
        && !rack_id.is_empty()
        && let Err(e) = validation_service.validate_rack_exists(rack_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(project_id) = &registration.project_id
        && !project_id.is_empty()
        && let Err(e) = validation_service.validate_project_exists(project_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(owner_id) = &registration.owner_id
        && !owner_id.is_empty()
        && let Err(e) = validation_service.validate_person_exists(owner_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    // Queue the registration message
    if let Err(err) = message_queue.send_message(Message::ClientRegistration(registration.clone()))
    {
        error!(
            "Failed to queue registration for {}: {}",
            registration.hostname, err
        );
        let response = ApiResponse::<Client> {
            status: err.status_code(),
            message: format!("Failed to queue registration: {}", err),
            data: None,
        };

        return (
            StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(response),
        );
    }

    // Process registration synchronously
    match client_service
        .register_client(
            &registration.hostname,
            &registration.ip_address,
            &registration.sys_vendor.unwrap_or_default(),
            &registration.product_name.unwrap_or_default(),
            &registration.serial_number.unwrap_or_default(),
            &registration.os.unwrap_or_default(),
            Some(registration.id.clone()),
        )
        .await
    {
        Ok(client) => {
            let response = ApiResponse {
                status: 200,
                message: "Client registered successfully".to_string(),
                data: Some(client),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!(
                "Failed to register client {}: {}",
                registration.hostname, err
            );
            let response = ApiResponse::<Client> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Delete a client
#[debug_handler]
#[instrument(skip(client_service, component_repo))]
pub async fn delete_client(
    Path(client_id): Path<String>,
    Extension(client_service): Extension<Arc<ClientService>>,
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
) -> impl IntoResponse {
    info!("Deleting client: {}", client_id);

    // Cascade update: Release components
    if let Err(e) = component_repo
        .release_components_by_client(&client_id)
        .await
    {
        let response = ApiResponse::<()> {
            status: 500,
            message: format!("Failed to release components: {}", e),
            data: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
    }

    match client_service.delete_client(&client_id).await {
        Ok(_) => {
            let response = ApiResponse::<()> {
                status: 200,
                message: "Client deleted successfully".to_string(),
                data: None,
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to delete client {}: {}", client_id, err);
            let response = ApiResponse::<()> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Update a client
#[debug_handler]
#[instrument(skip(client_repo, validation_service))]
pub async fn update_client(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Json(client): Json<Client>,
) -> impl IntoResponse {
    info!("Updating client: {}", client_id);

    // Validate references
    if let Some(rack_id) = &client.rack
        && !rack_id.is_empty()
        && let Err(e) = validation_service.validate_rack_exists(rack_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(project_id) = &client.project_id
        && !project_id.is_empty()
        && let Err(e) = validation_service.validate_project_exists(project_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(owner_id) = &client.owner_id
        && !owner_id.is_empty()
        && let Err(e) = validation_service.validate_person_exists(owner_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.to_string(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    // Check if client exists
    match client_repo.get(&client_id).await {
        Ok(Some(_existing_client)) => {
            // Ensure ID matches
            if client.id != client_id {
                let response = ApiResponse::<Client> {
                    status: 400,
                    message: "Client ID mismatch".to_string(),
                    data: None,
                };
                return (StatusCode::BAD_REQUEST, Json(response));
            }

            // Preserve some fields that shouldn't be overwritten by manual update if they are missing in request
            // But here we assume the frontend sends the full object or we overwrite.
            // For safety, let's ensure critical fields like registered_at are preserved if not provided (though Client struct has them)

            // Actually, we should probably merge or just trust the input.
            // Since this is an admin operation, we trust the input but preserve registered_at if it's somehow empty/defaulted?
            // The Client struct default has registered_at as Some(now).

            // Let's just save it. The frontend should have fetched the client first.

            match client_repo.save(&client).await {
                Ok(_) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Client updated successfully".to_string(),
                        data: Some(client),
                    };
                    (StatusCode::OK, Json(response))
                }
                Err(err) => {
                    error!("Failed to update client {}: {}", client_id, err);
                    let response = ApiResponse::<Client> {
                        status: err.status_code(),
                        message: err.to_string(),
                        data: None,
                    };
                    (
                        StatusCode::from_u16(err.status_code())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(response),
                    )
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<Client> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to check client {}: {}", client_id, err);
            let response = ApiResponse::<Client> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Search clients with advanced filtering
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn search_clients(
    Query(params): Query<SearchQuery>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    match client_repo.list_all().await {
        Ok(clients) => {
            let mut enriched_clients = Vec::new();

            // 获取每个客户端的硬件信息并进行搜索过滤
            for client in clients {
                let hardware = hardware_repo.get_hardware(&client.id).await.unwrap_or(None);

                // 检查是否匹配搜索条件
                let matches_search = if let Some(ref search_term) = params.q {
                    if search_term.is_empty() {
                        true
                    } else {
                        let search_lower = search_term.to_lowercase();

                        // 搜索客户端基本信息
                        let client_matches = client.hostname.to_lowercase().contains(&search_lower)
                            || client.ip_address.to_lowercase().contains(&search_lower)
                            || client
                                .os
                                .as_ref()
                                .is_some_and(|os| os.to_lowercase().contains(&search_lower))
                            || client
                                .sys_vendor
                                .as_ref()
                                .is_some_and(|v| v.to_lowercase().contains(&search_lower))
                            || client
                                .product_name
                                .as_ref()
                                .is_some_and(|p| p.to_lowercase().contains(&search_lower))
                            || client
                                .serial_number
                                .as_ref()
                                .is_some_and(|s| s.to_lowercase().contains(&search_lower));

                        // 搜索硬件信息
                        let hardware_matches = if let Some(ref hw) = hardware {
                            // 搜索操作系统信息
                            let os_matches =
                                hw.cpu.vendor_id.to_lowercase().contains(&search_lower)
                                    || hw.cpu.model_name.to_lowercase().contains(&search_lower);

                            // 搜索GPU信息
                            let gpu_matches = hw.gpus.iter().any(|gpu| {
                                gpu.vendor.to_lowercase().contains(&search_lower)
                                    || gpu.model.to_lowercase().contains(&search_lower)
                                    || gpu.device_id.to_lowercase().contains(&search_lower)
                                    || gpu.driver_version.to_lowercase().contains(&search_lower)
                            });

                            // 搜索内存信息
                            let ram_matches = hw.ram.vendor.to_lowercase().contains(&search_lower)
                                || hw.ram.model.to_lowercase().contains(&search_lower)
                                || hw.ram.form_factor.to_lowercase().contains(&search_lower)
                                || hw.ram.modules.iter().any(|module| {
                                    module.vendor.to_lowercase().contains(&search_lower)
                                        || module.part_number.to_lowercase().contains(&search_lower)
                                        || module.memory_type.to_lowercase().contains(&search_lower)
                                });

                            // 搜索磁盘信息
                            let disk_matches = hw.disks.iter().any(|disk| {
                                disk.vendor.to_lowercase().contains(&search_lower)
                                    || disk.model.to_lowercase().contains(&search_lower)
                                    || disk.serial_number.to_lowercase().contains(&search_lower)
                                    || disk.firmware_version.to_lowercase().contains(&search_lower)
                            });

                            // 搜索网卡信息
                            let nic_matches = hw.nics.iter().any(|nic| {
                                nic.name.to_lowercase().contains(&search_lower)
                                    || nic.vendor.to_lowercase().contains(&search_lower)
                                    || nic.model.to_lowercase().contains(&search_lower)
                                    || nic.mac_address.to_lowercase().contains(&search_lower)
                                    || nic.ipv4_address.to_lowercase().contains(&search_lower)
                                    || nic.ipv6_address.to_lowercase().contains(&search_lower)
                            });

                            os_matches || gpu_matches || ram_matches || disk_matches || nic_matches
                        } else {
                            false
                        };

                        client_matches || hardware_matches
                    }
                } else {
                    true
                };

                // 检查操作系统过滤
                let matches_os = if let Some(ref os_filter) = params.os {
                    if os_filter == "all" {
                        true
                    } else {
                        client.os.as_ref() == Some(os_filter)
                    }
                } else {
                    true
                };

                // 检查状态过滤
                let matches_status = if let Some(ref status_filter) = params.status {
                    if status_filter == "all" {
                        true
                    } else {
                        let is_online = client
                            .last_seen
                            .as_ref()
                            .and_then(|last_seen| {
                                chrono::DateTime::parse_from_rfc3339(last_seen).ok()
                            })
                            .map(|dt| {
                                let now = chrono::Utc::now();
                                let duration =
                                    now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                                duration.num_minutes() <= 5
                            })
                            .unwrap_or(false);

                        match status_filter.as_str() {
                            "online" => is_online,
                            "offline" => !is_online,
                            _ => true,
                        }
                    }
                } else {
                    true
                };

                if matches_search && matches_os && matches_status {
                    enriched_clients.push(client);
                }
            }

            info!("Search returned {} clients", enriched_clients.len());
            let response = ApiResponse {
                status: 200,
                message: "Clients searched successfully".to_string(),
                data: Some(enriched_clients),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to search clients: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Filter clients by hardware specifications
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn filter_clients_by_hardware(
    Query(params): Query<HardwareFilterQuery>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    match client_repo.list_all().await {
        Ok(clients) => {
            let mut filtered_clients = Vec::new();

            for client in clients {
                let hardware = hardware_repo.get_hardware(&client.id).await.unwrap_or(None);

                // Apply search term filter
                let matches_search = if let Some(ref search_term) = params.search_term {
                    if search_term.is_empty() {
                        true
                    } else {
                        let search_lower = search_term.to_lowercase();
                        client.hostname.to_lowercase().contains(&search_lower)
                            || client.ip_address.to_lowercase().contains(&search_lower)
                            || client
                                .os
                                .as_ref()
                                .is_some_and(|os| os.to_lowercase().contains(&search_lower))
                            || client
                                .sys_vendor
                                .as_ref()
                                .is_some_and(|v| v.to_lowercase().contains(&search_lower))
                            || client
                                .product_name
                                .as_ref()
                                .is_some_and(|p| p.to_lowercase().contains(&search_lower))
                            || client
                                .serial_number
                                .as_ref()
                                .is_some_and(|s| s.to_lowercase().contains(&search_lower))
                    }
                } else {
                    true
                };

                // Apply OS filter
                let matches_os = if let Some(ref os_filter) = params.os_filter {
                    if os_filter.is_empty() || os_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client.os.as_ref() == Some(os_filter)
                    }
                } else {
                    true
                };

                // Apply Kernel filter
                let matches_kernel = if let Some(ref kernel_filter) = params.os_kernel_filter {
                    if kernel_filter.is_empty() || kernel_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client.kernel_version.as_ref() == Some(kernel_filter)
                    }
                } else {
                    true
                };

                // Apply server vendor filter
                let matches_server_vendor = if let Some(ref vendor_filter) =
                    params.server_vendor_filter
                {
                    if vendor_filter.is_empty() || vendor_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client.sys_vendor.as_ref() == Some(vendor_filter)
                    }
                } else {
                    true
                };

                // Apply server model filter
                let matches_server_model =
                    if let Some(ref model_filter) = params.server_model_filter {
                        if model_filter.is_empty() || model_filter == crate::constants::FILTER_ALL {
                            true
                        } else {
                            client.product_name.as_ref() == Some(model_filter)
                        }
                    } else {
                        true
                    };

                // Apply status filter (online/offline)
                let matches_status = if let Some(ref status_filter) = params.status_filter {
                    if status_filter.is_empty() || status_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        let is_online = client
                            .last_seen
                            .as_ref()
                            .and_then(|last_seen| {
                                chrono::DateTime::parse_from_rfc3339(last_seen).ok()
                            })
                            .map(|dt| {
                                let now = chrono::Utc::now();
                                let duration =
                                    now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                                duration.num_minutes() <= 5
                            })
                            .unwrap_or(false);

                        match status_filter.as_str() {
                            "online" => is_online,
                            "offline" => !is_online,
                            _ => true,
                        }
                    }
                } else {
                    true
                };

                // Apply client status filter (Active, Maintenance, etc.)
                let matches_client_status = if let Some(ref status_filter) =
                    params.client_status_filter
                {
                    if status_filter.is_empty() || status_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client
                            .status
                            .as_ref()
                            .is_some_and(|s| format!("{:?}", s) == *status_filter)
                    }
                } else {
                    true
                };

                // Apply environment filter
                let matches_environment = if let Some(ref env_filter) = params.environment_filter {
                    if env_filter.is_empty() || env_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client
                            .environment
                            .as_ref()
                            .is_some_and(|e| format!("{:?}", e) == *env_filter)
                    }
                } else {
                    true
                };

                // Apply rack filter
                let matches_rack = if let Some(ref rack_filter) = params.rack_id_filter {
                    if rack_filter.is_empty() || rack_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client.rack.as_ref() == Some(rack_filter)
                    }
                } else {
                    true
                };

                // Apply project filter
                let matches_project = if let Some(ref project_filter) = params.project_id_filter {
                    if project_filter.is_empty() || project_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client.project_id.as_ref() == Some(project_filter)
                    }
                } else {
                    true
                };

                // Apply owner filter
                let matches_owner = if let Some(ref owner_filter) = params.owner_id_filter {
                    if owner_filter.is_empty() || owner_filter == crate::constants::FILTER_ALL {
                        true
                    } else {
                        client.owner_id.as_ref() == Some(owner_filter)
                    }
                } else {
                    true
                };

                // Apply hardware filters if hardware data is available
                let matches_hardware = if let Some(ref hw) = hardware {
                    // CPU vendor filter
                    let matches_cpu_vendor =
                        if let Some(ref cpu_vendor_filter) = params.cpu_vendor_filter {
                            if cpu_vendor_filter.is_empty()
                                || cpu_vendor_filter == crate::constants::FILTER_ALL
                            {
                                true
                            } else {
                                hw.cpu.vendor_id == *cpu_vendor_filter
                            }
                        } else {
                            true
                        };

                    // CPU model filter
                    let matches_cpu_model =
                        if let Some(ref cpu_model_filter) = params.cpu_model_filter {
                            if cpu_model_filter.is_empty()
                                || cpu_model_filter == crate::constants::FILTER_ALL
                            {
                                true
                            } else {
                                hw.cpu.model_name == *cpu_model_filter
                            }
                        } else {
                            true
                        };

                    // GPU vendor filter
                    let matches_gpu_vendor =
                        if let Some(ref gpu_vendor_filter) = params.gpu_vendor_filter {
                            if gpu_vendor_filter.is_empty()
                                || gpu_vendor_filter == crate::constants::FILTER_ALL
                            {
                                true
                            } else {
                                hw.gpus.iter().any(|gpu| gpu.vendor == *gpu_vendor_filter)
                            }
                        } else {
                            true
                        };

                    // GPU model filter
                    let matches_gpu_model =
                        if let Some(ref gpu_model_filter) = params.gpu_model_filter {
                            if gpu_model_filter.is_empty()
                                || gpu_model_filter == crate::constants::FILTER_ALL
                            {
                                true
                            } else {
                                hw.gpus.iter().any(|gpu| gpu.model == *gpu_model_filter)
                            }
                        } else {
                            true
                        };

                    // Memory range filter
                    let matches_memory = {
                        let memory_gb = hw.ram.total_size;
                        let matches_min = if let Some(min_memory) = params.memory_min_filter {
                            memory_gb >= min_memory
                        } else {
                            true
                        };
                        let matches_max = if let Some(max_memory) = params.memory_max_filter {
                            memory_gb <= max_memory
                        } else {
                            true
                        };
                        matches_min && matches_max
                    };

                    // Network type filter
                    let matches_network_type =
                        if let Some(ref network_type_filter) = params.network_type_filter {
                            if network_type_filter.is_empty()
                                || network_type_filter == crate::constants::FILTER_ALL
                            {
                                true
                            } else {
                                hw.nics
                                    .iter()
                                    .any(|nic| nic.nic_type.to_string() == *network_type_filter)
                            }
                        } else {
                            true
                        };

                    // Network model filter
                    let matches_network_model =
                        if let Some(ref network_model_filter) = params.network_model_filter {
                            if network_model_filter.is_empty()
                                || network_model_filter == crate::constants::FILTER_ALL
                            {
                                true
                            } else {
                                hw.nics.iter().any(|nic| nic.model == *network_model_filter)
                            }
                        } else {
                            true
                        };

                    // Storage type filter
                    let matches_storage =
                        if let Some(ref storage_type_filter) = params.storage_type_filter {
                            if storage_type_filter.is_empty()
                                || storage_type_filter == crate::constants::FILTER_ALL
                            {
                                true
                            } else {
                                hw.disks.iter().any(|disk| {
                                    disk.storage_type.to_string() == *storage_type_filter
                                })
                            }
                        } else {
                            true
                        };

                    matches_cpu_vendor
                        && matches_cpu_model
                        && matches_gpu_vendor
                        && matches_gpu_model
                        && matches_memory
                        && matches_network_type
                        && matches_network_model
                        && matches_storage
                } else {
                    // If no hardware data available, only exclude if hardware filters are set
                    // Allow devices without hardware data to pass through if no hardware filters are applied
                    let has_hardware_filters = params.cpu_vendor_filter.is_some()
                        || params.cpu_model_filter.is_some()
                        || params.gpu_vendor_filter.is_some()
                        || params.gpu_model_filter.is_some()
                        || params.memory_min_filter.is_some()
                        || params.memory_max_filter.is_some()
                        || params.network_type_filter.is_some()
                        || params.network_model_filter.is_some()
                        || params.storage_type_filter.is_some();

                    // 如果有硬件筛选条件但设备没有硬件数据，则排除该设备
                    // 如果没有硬件筛选条件，则允许通过
                    !has_hardware_filters
                };

                if matches_search
                    && matches_os
                    && matches_kernel
                    && matches_server_vendor
                    && matches_server_model
                    && matches_hardware
                    && matches_status
                    && matches_client_status
                    && matches_environment
                    && matches_rack
                    && matches_project
                    && matches_owner
                {
                    filtered_clients.push(client);
                }
            }

            info!(
                "Hardware filter returned {} clients",
                filtered_clients.len()
            );
            let response = ApiResponse {
                status: 200,
                message: crate::constants::MSG_CLIENTS_FILTERED_SUCCESS.to_string(),
                data: Some(filtered_clients),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to filter clients by hardware: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// 根据设备ID列表获取硬件筛选选项
#[debug_handler]
pub async fn get_filter_options_by_client_ids(
    Query(params): Query<HashMap<String, String>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    use common::models::{ApiResponse, FilterOptions};

    let client_ids_str = params.get("client_ids").unwrap_or(&String::new()).clone();

    if client_ids_str.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: crate::constants::MSG_EMPTY_CLIENT_IDS.to_string(),
            data: Some(FilterOptions::default()),
        };
        return (StatusCode::OK, Json(response));
    }

    // 解析client_ids字符串（逗号分隔）
    let client_ids: Vec<String> = client_ids_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if client_ids.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: crate::constants::MSG_NO_VALID_CLIENT_IDS.to_string(),
            data: Some(FilterOptions::default()),
        };
        return (StatusCode::OK, Json(response));
    }

    // 获取指定设备列表
    let mut clients = Vec::new();
    for client_id in &client_ids {
        match client_repo.get(client_id).await {
            Ok(Some(client)) => clients.push(client),
            Ok(None) => continue,
            Err(_) => continue,
        }
    }

    if clients.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: crate::constants::MSG_NO_CLIENTS_FOUND.to_string(),
            data: Some(FilterOptions::default()),
        };
        return (StatusCode::OK, Json(response));
    }

    // 获取这些设备的硬件信息
    let mut hardware_list = Vec::new();
    for client in &clients {
        match hardware_repo.get_hardware(&client.id).await {
            Ok(Some(hardware)) => hardware_list.push(hardware),
            Ok(None) => continue,
            Err(_) => continue,
        }
    }

    // 统计硬件选项
    let mut options = FilterOptions::default();
    let mut cpu_vendors = std::collections::HashSet::new();
    let mut cpu_models = std::collections::HashSet::new();
    let mut gpu_vendors = std::collections::HashSet::new();
    let mut gpu_models = std::collections::HashSet::new();
    let mut network_types = std::collections::HashSet::new();
    let mut network_models = std::collections::HashSet::new();
    let mut storage_types = std::collections::HashSet::new();
    // memory_ranges在FilterOptions中不存在，移除此变量

    for hardware in &hardware_list {
        // CPU信息
        if !hardware.cpu.vendor_id.is_empty() {
            cpu_vendors.insert(hardware.cpu.vendor_id.clone());
        }
        if !hardware.cpu.model_name.is_empty() {
            cpu_models.insert(hardware.cpu.model_name.clone());
        }

        // GPU信息
        for gpu in &hardware.gpus {
            if !gpu.vendor.is_empty() {
                gpu_vendors.insert(gpu.vendor.clone());
            }
            if !gpu.model.is_empty() {
                gpu_models.insert(gpu.model.clone());
            }
        }

        // 网络信息
        for nic in &hardware.nics {
            network_types.insert(nic.nic_type.to_string());
            if !nic.model.is_empty() {
                network_models.insert(nic.model.clone());
            }
        }

        // 存储信息
        for disk in &hardware.disks {
            storage_types.insert(disk.storage_type.to_string());
        }

        // 内存信息 - FilterOptions中暂时不包含memory_ranges字段
    }

    // 转换为Vector并排序
    options.cpu_vendors = {
        let mut vec: Vec<String> = cpu_vendors.into_iter().collect();
        vec.sort();
        vec
    };

    options.cpu_models = {
        let mut vec: Vec<String> = cpu_models.into_iter().collect();
        vec.sort();
        vec
    };

    options.gpu_vendors = {
        let mut vec: Vec<String> = gpu_vendors.into_iter().collect();
        vec.sort();
        vec
    };

    options.gpu_models = {
        let mut vec: Vec<String> = gpu_models.into_iter().collect();
        vec.sort();
        vec
    };

    options.network_types = {
        let mut vec: Vec<String> = network_types.into_iter().collect();
        vec.sort();
        vec
    };

    options.network_models = {
        let mut vec: Vec<String> = network_models.into_iter().collect();
        vec.sort();
        vec
    };

    options.storage_types = {
        let mut vec: Vec<String> = storage_types.into_iter().collect();
        vec.sort();
        vec
    };

    // memory_ranges字段在FilterOptions中不存在，这里移除

    let response = ApiResponse {
        status: 200,
        message: crate::constants::MSG_FILTER_OPTIONS_SUCCESS.to_string(),
        data: Some(options),
    };

    (StatusCode::OK, Json(response))
}

/// Export clients to JSON
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn export_clients(
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    match client_repo.list_all().await {
        Ok(clients) => (StatusCode::OK, Json(clients)),
        Err(err) => {
            error!("Failed to export clients: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

/// Import clients from JSON
#[debug_handler]
#[instrument(skip(client_service))]
pub async fn import_clients(
    Extension(client_service): Extension<Arc<ClientService>>,
    Json(clients): Json<Vec<Client>>,
) -> impl IntoResponse {
    info!("Importing {} clients", clients.len());
    match client_service.import_clients(clients).await {
        Ok(count) => {
            let response = ApiResponse {
                status: 200,
                message: format!("Successfully imported {} clients", count),
                data: Some(()),
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to import clients: {}", err);
            let response = ApiResponse::<()> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}
