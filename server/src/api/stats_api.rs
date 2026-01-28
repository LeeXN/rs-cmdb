//! Statistics API endpoints
//!
//! This module provides HTTP endpoints for hardware and client statistics.
//! Business logic has been moved to the StatsService.

use crate::repository::{
    client_repository::ClientRepository, hardware_repository::HardwareRepository,
};
use crate::service::stats_service::{CategoryStats as ServiceCategoryStats, OverallStats as ServiceOverallStats, StatsService};
use axum::{
    Json,
    extract::{Extension, Query},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use common::models::{
    ApiResponse, Client, ClientHardwareExport, DetailedStats, FilterCriteria,
    FilterOptions, StatItem,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsQuery {
    pub category: Option<String>, // cpu, memory, gpu, disk, nic, os, kernel, server_model
}

// Re-export types from service for API compatibility
#[derive(Debug, Serialize, PartialEq)]
pub struct CategoryStats {
    pub category: String,
    pub total_clients: usize,
    pub items: Vec<StatItem>,
}

impl From<ServiceCategoryStats> for CategoryStats {
    fn from(inner: ServiceCategoryStats) -> Self {
        Self {
            category: inner.category,
            total_clients: inner.total_clients,
            items: inner.items,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct OverallStats {
    pub total_clients: usize,
    pub online_clients: usize,
    pub offline_clients: usize,
    pub categories: Vec<CategoryStats>,
}

impl From<ServiceOverallStats> for OverallStats {
    fn from(inner: ServiceOverallStats) -> Self {
        Self {
            total_clients: inner.total_clients,
            online_clients: inner.online_clients,
            offline_clients: inner.offline_clients,
            categories: inner.categories.into_iter().map(CategoryStats::from).collect(),
        }
    }
}

/// Get hardware statistics
#[debug_handler]
#[instrument(skip(stats_service))]
pub async fn get_hardware_stats(
    Query(params): Query<StatsQuery>,
    Extension(stats_service): Extension<Arc<StatsService>>,
) -> impl IntoResponse {
    match stats_service
        .get_overall_stats(params.category.as_deref())
        .await
    {
        Ok(stats) => {
            let api_stats = OverallStats::from(stats);
            info!(
                "Generated hardware stats for {} clients",
                api_stats.total_clients
            );

            let response = ApiResponse {
                status: 200,
                message: "Hardware statistics retrieved successfully".to_string(),
                data: Some(api_stats),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to generate hardware stats: {}", err);
            let response = ApiResponse::<OverallStats> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Get detailed hardware statistics
#[debug_handler]
#[instrument(skip(stats_service))]
pub async fn get_detailed_stats(
    Extension(stats_service): Extension<Arc<StatsService>>,
) -> impl IntoResponse {
    match stats_service.get_detailed_stats().await {
        Ok(stats) => {
            info!("Generated detailed stats for {} clients", stats.total_clients);

            let response = ApiResponse {
                status: 200,
                message: "Detailed statistics retrieved successfully".to_string(),
                data: Some(stats),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to generate detailed stats: {}", err);
            let response = ApiResponse::<DetailedStats> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Get filter options from actual database data
#[debug_handler]
#[instrument(skip(stats_service))]
pub async fn get_filter_options(
    Extension(stats_service): Extension<Arc<StatsService>>,
) -> impl IntoResponse {
    match stats_service.get_filter_options().await {
        Ok(options) => {
            info!("Generated filter options");

            let response = ApiResponse {
                status: 200,
                message: "Filter options retrieved successfully".to_string(),
                data: Some(options),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to generate filter options: {}", err);
            let response = ApiResponse::<FilterOptions> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Get clients by specific hardware criteria
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn get_clients_by_criteria(
    Query(params): Query<HashMap<String, String>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for criteria: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            return (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            );
        }
    };

    // Filter clients based on criteria
    let filtered_clients = if let Some(ids_str) = params.get("ids") {
        let ids: Vec<&str> = ids_str.split(',').collect();
        clients
            .into_iter()
            .filter(|c| ids.contains(&c.id.as_str()))
            .collect()
    } else {
        clients
    };

    info!("Retrieved {} clients by criteria", filtered_clients.len());

    let response = ApiResponse {
        status: 200,
        message: "Clients retrieved successfully".to_string(),
        data: Some(filtered_clients),
    };

    (StatusCode::OK, Json(response))
}

/// Filter clients by criteria
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn filter_clients(
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
    Json(filter): Json<FilterCriteria>,
) -> impl IntoResponse {
    // Get all clients
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for filtering: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            return (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            );
        }
    };

    // Filter clients based on criteria
    let mut filtered_clients = Vec::new();

    for client in clients {
        let mut matches = true;

        // Check OS filter
        if let Some(ref os_name) = filter.os_name {
            if let Some(ref client_os) = client.os {
                if !client_os.to_lowercase().contains(&os_name.to_lowercase()) {
                    matches = false;
                }
            } else {
                matches = false;
            }
        }

        // Check Kernel filter
        if let Some(ref os_kernel) = filter.os_kernel {
            if let Some(ref client_kernel) = client.kernel_version {
                if !client_kernel
                    .to_lowercase()
                    .contains(&os_kernel.to_lowercase())
                {
                    matches = false;
                }
            } else {
                matches = false;
            }
        }

        // Check server vendor filter
        if let Some(ref server_vendor) = filter.server_vendor {
            if let Some(ref client_vendor) = client.sys_vendor {
                if !client_vendor
                    .to_lowercase()
                    .contains(&server_vendor.to_lowercase())
                {
                    matches = false;
                }
            } else {
                matches = false;
            }
        }

        // Check hardware-based filters
        if matches
            && (filter.cpu_vendor.is_some()
                || filter.cpu_model.is_some()
                || filter.cpu_cores.is_some()
                || filter.memory_capacity_min.is_some()
                || filter.memory_capacity_max.is_some()
                || filter.gpu_vendor.is_some()
                || filter.gpu_model.is_some()
                || filter.storage_type.is_some()
                || filter.network_type.is_some())
        {
            if let Ok(Some(hardware)) = hardware_repo.get_hardware(&client.id).await {
                // Check CPU filters
                if let Some(ref cpu_vendor) = filter.cpu_vendor
                    && !hardware
                        .cpu
                        .vendor_id
                        .to_lowercase()
                        .contains(&cpu_vendor.to_lowercase())
                {
                    matches = false;
                }

                if let Some(ref cpu_model) = filter.cpu_model
                    && !hardware
                        .cpu
                        .model_name
                        .to_lowercase()
                        .contains(&cpu_model.to_lowercase())
                {
                    matches = false;
                }

                if let Some(cpu_cores) = filter.cpu_cores
                    && hardware.cpu.cores != cpu_cores
                {
                    matches = false;
                }

                // Check memory filters
                if let Some(min_capacity) = filter.memory_capacity_min
                    && hardware.ram.total_size < min_capacity
                {
                    matches = false;
                }

                if let Some(max_capacity) = filter.memory_capacity_max
                    && hardware.ram.total_size > max_capacity
                {
                    matches = false;
                }

                // Check GPU filters
                if let Some(ref gpu_vendor) = filter.gpu_vendor {
                    let has_matching_gpu = hardware.gpus.iter().any(|gpu| {
                        gpu.vendor
                            .to_lowercase()
                            .contains(&gpu_vendor.to_lowercase())
                    });
                    if !has_matching_gpu {
                        matches = false;
                    }
                }

                // Check GPU model filter
                if let Some(ref gpu_model) = filter.gpu_model {
                    let has_matching_gpu_model = hardware
                        .gpus
                        .iter()
                        .any(|gpu| gpu.model.to_lowercase().contains(&gpu_model.to_lowercase()));
                    if !has_matching_gpu_model {
                        matches = false;
                    }
                }

                // Check storage type filter
                if let Some(ref storage_type) = filter.storage_type {
                    let has_matching_storage = hardware.disks.iter().any(|disk| {
                        format!("{:?}", disk.storage_type)
                            .to_lowercase()
                            .contains(&storage_type.to_lowercase())
                    });
                    if !has_matching_storage {
                        matches = false;
                    }
                }

                // Check network type filter
                if let Some(ref network_type) = filter.network_type {
                    let has_matching_network = hardware.nics.iter().any(|nic| {
                        nic.nic_type
                            .to_string()
                            .to_lowercase()
                            .contains(&network_type.to_lowercase())
                    });
                    if !has_matching_network {
                        matches = false;
                    }
                }
            } else {
                matches = false;
            }
        }

        if matches {
            filtered_clients.push(client);
        }
    }

    info!("Filtered clients: {} matches found", filtered_clients.len());

    let response = ApiResponse {
        status: 200,
        message: "Clients filtered successfully".to_string(),
        data: Some(filtered_clients),
    };

    (StatusCode::OK, Json(response))
}

/// Export detailed client and hardware data
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn export_client_hardware_data(
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    // Get all clients
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for export: {}", err);
            let response = ApiResponse::<Vec<ClientHardwareExport>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            return (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            );
        }
    };

    let mut export_data = Vec::new();

    for client in clients {
        let hardware = hardware_repo.get_hardware(&client.id).await.unwrap_or(None);

        let export_item = ClientHardwareExport {
            // 基本信息
            client_id: client.id.clone(),
            hostname: client.hostname.clone(),
            ip_address: client.ip_address.clone(),
            os: client
                .os
                .clone()
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            kernel_version: client
                .kernel_version
                .clone()
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            sys_vendor: client
                .sys_vendor
                .clone()
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            product_name: client
                .product_name
                .clone()
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            serial_number: client
                .serial_number
                .clone()
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            last_seen: client
                .last_seen
                .clone()
                .unwrap_or_else(|| crate::constants::NEVER_SEEN.to_string()),
            registered_at: client
                .registered_at
                .clone()
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),

            // 硬件信息
            cpu_vendor: hardware
                .as_ref()
                .map(|h| h.cpu.vendor_id.clone())
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            cpu_model: hardware
                .as_ref()
                .map(|h| h.cpu.model_name.clone())
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            cpu_cores: hardware.as_ref().map(|h| h.cpu.cores).unwrap_or(0),
            cpu_threads: hardware.as_ref().map(|h| h.cpu.threads).unwrap_or(0),
            cpu_frequency: hardware
                .as_ref()
                .map(|h| {
                    format!(
                        "{:.2} {}",
                        h.cpu.speed as f64 / 1000.0,
                        crate::constants::UNIT_GHZ
                    )
                })
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),

            // 内存信息
            memory_total: hardware
                .as_ref()
                .map(|h| format!("{}{}", h.ram.total_size, crate::constants::UNIT_GB))
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            memory_vendor: hardware
                .as_ref()
                .map(|h| h.ram.vendor.clone())
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            memory_speed: hardware
                .as_ref()
                .map(|h| format!("{}{}", h.ram.speed, crate::constants::UNIT_MHZ))
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            memory_modules: hardware
                .as_ref()
                .map(|h| h.ram.modules.len() as u32)
                .unwrap_or(0),

            // GPU信息
            gpu_count: hardware.as_ref().map(|h| h.gpus.len() as u32).unwrap_or(0),
            gpu_models: hardware
                .as_ref()
                .map(|h| {
                    if h.gpus.is_empty() {
                        crate::constants::UNKNOWN_GPU.to_string()
                    } else {
                        h.gpus
                            .iter()
                            .map(|gpu| gpu.model.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                })
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            gpu_vendors: hardware
                .as_ref()
                .map(|h| {
                    if h.gpus.is_empty() {
                        crate::constants::COUNT_NONE.to_string()
                    } else {
                        let mut vendors: Vec<String> =
                            h.gpus.iter().map(|gpu| gpu.vendor.clone()).collect();
                        vendors.sort();
                        vendors.dedup();
                        vendors.join(", ")
                    }
                })
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),

            // 存储信息
            storage_count: hardware.as_ref().map(|h| h.disks.len() as u32).unwrap_or(0),
            storage_total: hardware
                .as_ref()
                .map(|h| {
                    let total: f64 = h
                        .disks
                        .iter()
                        .filter_map(|d| d.size.parse::<f64>().ok())
                        .sum();
                    if total >= 1000.0 {
                        format!("{:.1}TB", total / 1000.0)
                    } else {
                        format!("{:.0}GB", total)
                    }
                })
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            storage_types: hardware
                .as_ref()
                .map(|h| {
                    if h.disks.is_empty() {
                        crate::constants::COUNT_NONE.to_string()
                    } else {
                        let mut types: Vec<String> = h
                            .disks
                            .iter()
                            .map(|disk| disk.storage_type.to_string())
                            .collect();
                        types.sort();
                        types.dedup();
                        types.join(", ")
                    }
                })
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),

            // 网络信息
            network_count: hardware.as_ref().map(|h| h.nics.len() as u32).unwrap_or(0),
            network_types: hardware
                .as_ref()
                .map(|h| {
                    if h.nics.is_empty() {
                        crate::constants::COUNT_NONE.to_string()
                    } else {
                        let mut types: Vec<String> =
                            h.nics.iter().map(|nic| nic.nic_type.to_string()).collect();
                        types.sort();
                        types.dedup();
                        types.join(", ")
                    }
                })
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            network_speeds: hardware
                .as_ref()
                .map(|h| {
                    if h.nics.is_empty() {
                        crate::constants::COUNT_NONE.to_string()
                    } else {
                        let speeds: Vec<String> = h
                            .nics
                            .iter()
                            .map(|nic| format!("{}Mbps", nic.speed))
                            .collect();
                        speeds.join(", ")
                    }
                })
                .unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
        };

        export_data.push(export_item);
    }

    info!("Exported data for {} clients", export_data.len());

    let response = ApiResponse {
        status: 200,
        message: "Export data retrieved successfully".to_string(),
        data: Some(export_data),
    };

    (StatusCode::OK, Json(response))
}
