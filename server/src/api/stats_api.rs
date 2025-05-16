use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use common::models::{ApiResponse, Client, DetailedStats, FilterCriteria, FilterOptions, StatItem, CpuStats, MemoryStats, GpuStats, NetworkStats, OsStats, ServerStats, StorageStats, ClientHardwareExport};
use common::entity::hardware::Hardware;
use crate::repository::{client_repository::ClientRepository, hardware_repository::HardwareRepository};
use tracing::{info, error, instrument};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsQuery {
    pub category: Option<String>, // cpu, memory, gpu, disk, nic, os, kernel, server_model
}

#[derive(Debug, Serialize, PartialEq)]
pub struct CategoryStats {
    pub category: String,
    pub total_clients: usize,
    pub items: Vec<StatItem>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct OverallStats {
    pub total_clients: usize,
    pub online_clients: usize,
    pub offline_clients: usize,
    pub categories: Vec<CategoryStats>,
}

/// Get hardware statistics
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn get_hardware_stats(
    Query(params): Query<StatsQuery>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    // Get all clients
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for stats: {}", err);
            let response = ApiResponse::<OverallStats> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            return (StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(response));
        }
    };

    let total_clients = clients.len();
    let online_clients = clients.iter().filter(|c| {
        c.last_seen.as_ref()
            .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
            .map(|dt| {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                duration.num_minutes() <= 5
            })
            .unwrap_or(false)
    }).count();
    let offline_clients = total_clients - online_clients;

    // Collect hardware data for all clients
    let mut client_hardware_map = HashMap::new();
    for client in &clients {
        if let Ok(Some(hardware)) = hardware_repo.get_hardware(&client.id).await {
            client_hardware_map.insert(client.id.clone(), (client.clone(), hardware));
        }
    }

    let mut categories = Vec::new();

    // Generate stats based on requested category or all categories
    match params.category.as_deref() {
        Some("cpu") => {
            categories.push(generate_cpu_stats(&client_hardware_map));
        },
        Some("memory") => {
            categories.push(generate_memory_stats(&client_hardware_map));
        },
        Some("gpu") => {
            categories.push(generate_gpu_stats(&client_hardware_map));
        },
        Some("disk") => {
            categories.push(generate_disk_stats(&client_hardware_map));
        },
        Some("nic") => {
            categories.push(generate_nic_stats(&client_hardware_map));
        },
        Some("os") => {
            categories.push(generate_os_stats(&clients));
        },
        Some("server_model") => {
            categories.push(generate_server_model_stats(&clients));
        },
        _ => {
            // Generate all categories
            categories.push(generate_cpu_stats(&client_hardware_map));
            categories.push(generate_memory_stats(&client_hardware_map));
            categories.push(generate_gpu_stats(&client_hardware_map));
            categories.push(generate_disk_stats(&client_hardware_map));
            categories.push(generate_nic_stats(&client_hardware_map));
            categories.push(generate_os_stats(&clients));
            categories.push(generate_server_model_stats(&clients));
        }
    }

    let stats = OverallStats {
        total_clients,
        online_clients,
        offline_clients,
        categories,
    };

    info!("Generated hardware stats for {} clients", total_clients);

    let response = ApiResponse {
        status: 200,
        message: "Hardware statistics retrieved successfully".to_string(),
        data: Some(stats),
    };

    (StatusCode::OK, Json(response))
}

/// Get clients by specific hardware criteria
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn get_clients_by_criteria(
    Query(params): Query<HashMap<String, String>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
) -> impl IntoResponse {
    // For now, just return all clients if no specific criteria handling
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for criteria: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            return (StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(response));
        }
    };

    // Filter clients based on criteria
    let filtered_clients = if let Some(ids_str) = params.get("ids") {
        let ids: Vec<&str> = ids_str.split(',').collect();
        clients.into_iter().filter(|c| ids.contains(&c.id.as_str())).collect()
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

// Helper functions to generate statistics for different categories

fn generate_cpu_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> CategoryStats {
    let mut cpu_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        let cpu_name = format!("{} ({} {})", hardware.cpu.model_name, hardware.cpu.cores, crate::constants::UNIT_CORES);
        cpu_stats.entry(cpu_name).or_insert_with(Vec::new).push(client.id.clone());
    }

            generate_category_stats(crate::constants::CATEGORY_CPU_CONFIG, cpu_stats, client_hardware_map.len())
}

fn generate_memory_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> CategoryStats {
    let mut memory_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        let memory_name = format!("{}GB", hardware.ram.total_size);
        memory_stats.entry(memory_name).or_insert_with(Vec::new).push(client.id.clone());
    }

            generate_category_stats(crate::constants::CATEGORY_MEMORY_CONFIG, memory_stats, client_hardware_map.len())
}

fn generate_gpu_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> CategoryStats {
    let mut gpu_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        if !hardware.gpus.is_empty() {
            let gpu_name = hardware.gpus[0].model.clone();
            gpu_stats.entry(gpu_name).or_insert_with(Vec::new).push(client.id.clone());
        } else {
            gpu_stats.entry(crate::constants::UNKNOWN_GPU.to_string()).or_insert_with(Vec::new).push(client.id.clone());
        }
    }

            generate_category_stats(crate::constants::CATEGORY_GPU_CONFIG, gpu_stats, client_hardware_map.len())
}

fn generate_disk_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> CategoryStats {
    let mut disk_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        if !hardware.disks.is_empty() {
            // 计算总存储大小，需要解析size字符串
            let total_size: f64 = hardware.disks.iter()
                .filter_map(|d| d.size.parse::<f64>().ok())
                .sum();
            let disk_name = format!("{:.0}GB", total_size);
            disk_stats.entry(disk_name).or_insert_with(Vec::new).push(client.id.clone());
        }
    }

            generate_category_stats(crate::constants::CATEGORY_STORAGE_CONFIG, disk_stats, client_hardware_map.len())
}

fn generate_nic_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> CategoryStats {
    let mut nic_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        let nic_count = hardware.nics.len();
        let nic_name = format!("{} {}", nic_count, crate::constants::COUNT_NICS);
        nic_stats.entry(nic_name).or_insert_with(Vec::new).push(client.id.clone());
    }

            generate_category_stats(crate::constants::CATEGORY_NETWORK_CONFIG, nic_stats, client_hardware_map.len())
}

fn generate_os_stats(clients: &[Client]) -> CategoryStats {
    let mut os_stats = HashMap::new();
    
    for client in clients {
        let os_name = client.os.as_ref().unwrap_or(&crate::constants::UNKNOWN_SYSTEM.to_string()).clone();
        os_stats.entry(os_name).or_insert_with(Vec::new).push(client.id.clone());
    }

            generate_category_stats(crate::constants::CATEGORY_OS, os_stats, clients.len())
}

fn generate_server_model_stats(clients: &[Client]) -> CategoryStats {
    let mut model_stats = HashMap::new();
    
    for client in clients {
        let model_name = if let (Some(vendor), Some(product)) = (&client.sys_vendor, &client.product_name) {
            format!("{} {}", vendor, product)
        } else {
            crate::constants::UNKNOWN_MODEL.to_string()
        };
        model_stats.entry(model_name).or_insert_with(Vec::new).push(client.id.clone());
    }

            generate_category_stats(crate::constants::CATEGORY_SERVER_MODEL, model_stats, clients.len())
}

fn generate_category_stats(category_name: &str, stats_map: HashMap<String, Vec<String>>, total_clients: usize) -> CategoryStats {
    let mut items: Vec<StatItem> = stats_map.into_iter().map(|(name, client_ids)| {
        let count = client_ids.len();
        let percentage = if total_clients > 0 {
            (count as f64 / total_clients as f64) * 100.0
        } else {
            0.0
        };
        
        StatItem {
            name,
            count,
            percentage,
            client_ids,
        }
    }).collect();

    // Sort by count in descending order
    items.sort_by(|a, b| b.count.cmp(&a.count));

    CategoryStats {
        category: category_name.to_string(),
        total_clients,
        items,
    }
}

/// Get detailed hardware statistics
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn get_detailed_stats(
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    // Get all clients
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for detailed stats: {}", err);
            let response = ApiResponse::<DetailedStats> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            return (StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(response));
        }
    };

    let total_clients = clients.len();
    let online_clients = clients.iter().filter(|c| {
        c.last_seen.as_ref()
            .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
            .map(|dt| {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                duration.num_minutes() <= 5
            })
            .unwrap_or(false)
    }).count();
    let offline_clients = total_clients - online_clients;

    // Collect hardware data for all clients
    let mut client_hardware_map = HashMap::new();
    for client in &clients {
        if let Ok(Some(hardware)) = hardware_repo.get_hardware(&client.id).await {
            client_hardware_map.insert(client.id.clone(), (client.clone(), hardware));
        }
    }

    // Generate detailed statistics
    let cpu_stats = generate_detailed_cpu_stats(&client_hardware_map);
    let memory_stats = generate_detailed_memory_stats(&client_hardware_map);
    let gpu_stats = generate_detailed_gpu_stats(&client_hardware_map);
    let network_stats = generate_detailed_network_stats(&client_hardware_map);
    let os_stats = generate_detailed_os_stats(&clients, &client_hardware_map);
    let server_stats = generate_detailed_server_stats(&clients);
    let storage_stats = generate_detailed_storage_stats(&client_hardware_map);

    let detailed_stats = DetailedStats {
        total_clients,
        online_clients,
        offline_clients,
        cpu_stats,
        memory_stats,
        gpu_stats,
        network_stats,
        os_stats,
        server_stats,
        storage_stats,
    };

    info!("Generated detailed stats for {} clients", total_clients);

    let response = ApiResponse {
        status: 200,
        message: "Detailed statistics retrieved successfully".to_string(),
        data: Some(detailed_stats),
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
            return (StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(response));
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
                if !client_kernel.to_lowercase().contains(&os_kernel.to_lowercase()) {
                    matches = false;
                }
            } else {
                matches = false;
            }
        }
        
        // Check server vendor filter
        if let Some(ref server_vendor) = filter.server_vendor {
            if let Some(ref client_vendor) = client.sys_vendor {
                if !client_vendor.to_lowercase().contains(&server_vendor.to_lowercase()) {
                    matches = false;
                }
            } else {
                matches = false;
            }
        }
        
        // Check hardware-based filters
        if matches && (filter.cpu_vendor.is_some() || filter.cpu_model.is_some() || filter.cpu_cores.is_some() ||
                      filter.memory_capacity_min.is_some() || filter.memory_capacity_max.is_some() ||
                      filter.gpu_vendor.is_some() || filter.gpu_model.is_some() || filter.storage_type.is_some() || filter.network_type.is_some()) {
            
            if let Ok(Some(hardware)) = hardware_repo.get_hardware(&client.id).await {
                // Check CPU filters
                if let Some(ref cpu_vendor) = filter.cpu_vendor {
                    if !hardware.cpu.vendor_id.to_lowercase().contains(&cpu_vendor.to_lowercase()) {
                        matches = false;
                    }
                }
                
                if let Some(ref cpu_model) = filter.cpu_model {
                    if !hardware.cpu.model_name.to_lowercase().contains(&cpu_model.to_lowercase()) {
                        matches = false;
                    }
                }
                
                if let Some(cpu_cores) = filter.cpu_cores {
                    if hardware.cpu.cores != cpu_cores {
                        matches = false;
                    }
                }
                
                // Check memory filters
                if let Some(min_capacity) = filter.memory_capacity_min {
                    if hardware.ram.total_size < min_capacity {
                        matches = false;
                    }
                }
                
                if let Some(max_capacity) = filter.memory_capacity_max {
                    if hardware.ram.total_size > max_capacity {
                        matches = false;
                    }
                }
                
                // Check GPU filters
                if let Some(ref gpu_vendor) = filter.gpu_vendor {
                    let has_matching_gpu = hardware.gpus.iter().any(|gpu| {
                        gpu.vendor.to_lowercase().contains(&gpu_vendor.to_lowercase())
                    });
                    if !has_matching_gpu {
                        matches = false;
                    }
                }
                
                // Check GPU model filter
                if let Some(ref gpu_model) = filter.gpu_model {
                    let has_matching_gpu_model = hardware.gpus.iter().any(|gpu| {
                        gpu.model.to_lowercase().contains(&gpu_model.to_lowercase())
                    });
                    if !has_matching_gpu_model {
                        matches = false;
                    }
                }
                
                // Check storage type filter
                if let Some(ref storage_type) = filter.storage_type {
                    let has_matching_storage = hardware.disks.iter().any(|disk| {
                        format!("{:?}", disk.storage_type).to_lowercase().contains(&storage_type.to_lowercase())
                    });
                    if !has_matching_storage {
                        matches = false;
                    }
                }
                
                // Check network type filter
                if let Some(ref network_type) = filter.network_type {
                    let has_matching_network = hardware.nics.iter().any(|nic| {
                        nic.nic_type.to_string().to_lowercase().contains(&network_type.to_lowercase())
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

/// Get filter options from actual database data
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn get_filter_options(
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    // Get all clients
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for filter options: {}", err);
            let response = ApiResponse::<FilterOptions> {
                status: err.status_code(),
                message: err.to_string(),
                data: None,
            };
            return (StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(response));
        }
    };

    // Collect unique values for each filter category
    let mut cpu_vendors = std::collections::HashSet::new();
    let mut cpu_models = std::collections::HashSet::new();
    let mut gpu_vendors = std::collections::HashSet::new();
    let mut gpu_models = std::collections::HashSet::new();
    let mut os_names = std::collections::HashSet::new();
    let mut os_kernels = std::collections::HashSet::new();
    let mut server_vendors = std::collections::HashSet::new();
    let mut storage_types = std::collections::HashSet::new();
    let mut network_types = std::collections::HashSet::new();
    let mut network_models = std::collections::HashSet::new();

    // Process client data
    for client in &clients {
        // OS names
        if let Some(ref os) = client.os {
            if !os.trim().is_empty() {
                os_names.insert(os.clone());
            }
        }

        // OS kernels
        if let Some(ref kernel) = client.kernel_version {
            if !kernel.trim().is_empty() {
                os_kernels.insert(kernel.clone());
            }
        }

        // Server vendors
        if let Some(ref vendor) = client.sys_vendor {
            if !vendor.trim().is_empty() {
                server_vendors.insert(vendor.clone());
            }
        }
    }

    // Process hardware data
    for client in &clients {
        if let Ok(Some(hardware)) = hardware_repo.get_hardware(&client.id).await {
            // CPU data
            if !hardware.cpu.vendor_id.trim().is_empty() {
                cpu_vendors.insert(hardware.cpu.vendor_id.clone());
            }
            if !hardware.cpu.model_name.trim().is_empty() {
                cpu_models.insert(hardware.cpu.model_name.clone());
            }

            // GPU data
            for gpu in &hardware.gpus {
                if !gpu.vendor.trim().is_empty() {
                    gpu_vendors.insert(gpu.vendor.clone());
                }
                if !gpu.model.trim().is_empty() {
                    gpu_models.insert(gpu.model.clone());
                }
            }

            // Storage data
            for disk in &hardware.disks {
                let storage_type = disk.storage_type.to_string();
                if !storage_type.trim().is_empty() {
                    storage_types.insert(storage_type);
                }
            }

            // Network data
            for nic in &hardware.nics {
                let network_type = nic.nic_type.to_string();
                if !network_type.trim().is_empty() {
                    network_types.insert(network_type);
                }
                if !nic.model.trim().is_empty() {
                    network_models.insert(nic.model.clone());
                }
            }
        }
    }

    // Convert HashSets to sorted Vectors
    let mut filter_options = FilterOptions {
        cpu_vendors: cpu_vendors.into_iter().collect(),
        cpu_models: cpu_models.into_iter().collect(),
        gpu_vendors: gpu_vendors.into_iter().collect(),
        gpu_models: gpu_models.into_iter().collect(),
        os_names: os_names.into_iter().collect(),
        os_kernels: os_kernels.into_iter().collect(),
        server_vendors: server_vendors.into_iter().collect(),
        storage_types: storage_types.into_iter().collect(),
        network_types: network_types.into_iter().collect(),
        network_models: network_models.into_iter().collect(),
    };

    // Sort all vectors for consistent ordering
    filter_options.cpu_vendors.sort();
    filter_options.cpu_models.sort();
    filter_options.gpu_vendors.sort();
    filter_options.gpu_models.sort();
    filter_options.os_names.sort();
    filter_options.os_kernels.sort();
    filter_options.server_vendors.sort();
    filter_options.storage_types.sort();
    filter_options.network_types.sort();
    filter_options.network_models.sort();

    info!("Generated filter options");

    let response = ApiResponse {
        status: 200,
        message: "Filter options retrieved successfully".to_string(),
        data: Some(filter_options),
    };

    (StatusCode::OK, Json(response))
}

// Helper functions for detailed statistics

fn generate_detailed_cpu_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> CpuStats {
    let mut vendor_stats = HashMap::new();
    let mut model_stats = HashMap::new();
    let mut cores_stats = HashMap::new();
    let mut threads_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        // By vendor
        vendor_stats.entry(hardware.cpu.vendor_id.clone()).or_insert_with(Vec::new).push(client.id.clone());
        
        // By model
        model_stats.entry(hardware.cpu.model_name.clone()).or_insert_with(Vec::new).push(client.id.clone());
        
        // By cores
        let cores_key = format!("{} {}", hardware.cpu.cores, crate::constants::UNIT_CORES);
        cores_stats.entry(cores_key).or_insert_with(Vec::new).push(client.id.clone());
        
        // By threads
        let threads_key = format!("{} {}", hardware.cpu.threads, crate::constants::UNIT_THREADS);
        threads_stats.entry(threads_key).or_insert_with(Vec::new).push(client.id.clone());
    }
    
    let total_clients = client_hardware_map.len();
    
    CpuStats {
        by_vendor: generate_stat_items(vendor_stats, total_clients),
        by_model: generate_stat_items(model_stats, total_clients),
        by_cores: generate_stat_items(cores_stats, total_clients),
        by_threads: generate_stat_items(threads_stats, total_clients),
    }
}

fn generate_detailed_memory_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> MemoryStats {
    let mut capacity_stats = HashMap::new();
    let mut vendor_stats = HashMap::new();
    let mut type_stats = HashMap::new();
    let mut speed_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        // By capacity
        let capacity_key = format!("{}GB", hardware.ram.total_size);
        capacity_stats.entry(capacity_key).or_insert_with(Vec::new).push(client.id.clone());
        
        // By vendor
        vendor_stats.entry(hardware.ram.vendor.clone()).or_insert_with(Vec::new).push(client.id.clone());
        
        // By type (from modules)
        if !hardware.ram.modules.is_empty() {
            let memory_type = hardware.ram.modules[0].memory_type.clone();
            type_stats.entry(memory_type).or_insert_with(Vec::new).push(client.id.clone());
        }
        
        // By speed
        let speed_key = format!("{}MHz", hardware.ram.speed);
        speed_stats.entry(speed_key).or_insert_with(Vec::new).push(client.id.clone());
    }
    
    let total_clients = client_hardware_map.len();
    
    MemoryStats {
        by_capacity: generate_stat_items(capacity_stats, total_clients),
        by_vendor: generate_stat_items(vendor_stats, total_clients),
        by_type: generate_stat_items(type_stats, total_clients),
        by_speed: generate_stat_items(speed_stats, total_clients),
    }
}

fn generate_detailed_gpu_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> GpuStats {
    let mut vendor_stats = HashMap::new();
    let mut model_stats = HashMap::new();
    let mut model_with_count_stats = HashMap::new();
    let mut driver_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        if hardware.gpus.is_empty() {
                    vendor_stats.entry(crate::constants::UNKNOWN_GPU.to_string()).or_insert_with(Vec::new).push(client.id.clone());
        model_stats.entry(crate::constants::UNKNOWN_GPU.to_string()).or_insert_with(Vec::new).push(client.id.clone());
        model_with_count_stats.entry(crate::constants::UNKNOWN_GPU.to_string()).or_insert_with(Vec::new).push(client.id.clone());
        driver_stats.entry(crate::constants::UNKNOWN_DRIVER.to_string()).or_insert_with(Vec::new).push(client.id.clone());
        } else {
            // 按机器统计GPU厂商和驱动（使用第一个GPU代表）
            let primary_gpu = &hardware.gpus[0];
            vendor_stats.entry(primary_gpu.vendor.clone()).or_insert_with(Vec::new).push(client.id.clone());
            driver_stats.entry(primary_gpu.driver_version.clone()).or_insert_with(Vec::new).push(client.id.clone());
            
            // 按GPU型号统计（不考虑数量，有就算）
            let mut unique_models = std::collections::HashSet::new();
            for gpu in &hardware.gpus {
                unique_models.insert(gpu.model.clone());
            }
            for model in unique_models {
                model_stats.entry(model).or_insert_with(Vec::new).push(client.id.clone());
            }
            
            // 按GPU型号和数量统计（详细统计）
            let mut model_counts = std::collections::HashMap::new();
            for gpu in &hardware.gpus {
                *model_counts.entry(gpu.model.clone()).or_insert(0) += 1;
            }
            
            for (model, count) in model_counts {
                let model_with_count_key = if count == 1 {
                    model
                } else {
                    format!("{}*{}", model, count)
                };
                model_with_count_stats.entry(model_with_count_key).or_insert_with(Vec::new).push(client.id.clone());
            }
        }
    }
    
    let total_clients = client_hardware_map.len();
    
    GpuStats {
        by_vendor: generate_stat_items(vendor_stats, total_clients),
        by_model: generate_stat_items(model_stats, total_clients),
        by_model_with_count: generate_stat_items(model_with_count_stats, total_clients),
        by_driver_version: generate_stat_items(driver_stats, total_clients),
    }
}

fn generate_detailed_network_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> NetworkStats {
    let mut type_stats = HashMap::new();
    let mut vendor_stats = HashMap::new();
    let mut speed_stats = HashMap::new();
    let mut status_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        if !hardware.nics.is_empty() {
            // 按机器统计，使用主要网卡类型
            let primary_nic = &hardware.nics[0];
            
            // By type
            let type_key = primary_nic.nic_type.to_string();
            type_stats.entry(type_key).or_insert_with(Vec::new).push(client.id.clone());
            
            // By vendor
            vendor_stats.entry(primary_nic.vendor.clone()).or_insert_with(Vec::new).push(client.id.clone());
            
            // By speed (使用最高速度)
            let max_speed = hardware.nics.iter().map(|nic| nic.speed).max().unwrap_or(0);
            let speed_key = format!("{}Mbps", max_speed);
            speed_stats.entry(speed_key).or_insert_with(Vec::new).push(client.id.clone());
            
            // By status (如果有任何网卡在线就算在线)
            let has_up_nic = hardware.nics.iter().any(|nic| matches!(nic.status, common::entity::hardware::NICStatus::Up));
            let status_key = if has_up_nic { "Up".to_string() } else { "Down".to_string() };
            status_stats.entry(status_key).or_insert_with(Vec::new).push(client.id.clone());
        }
    }
    
    let total_clients = client_hardware_map.len();
    
    NetworkStats {
        by_type: generate_stat_items(type_stats, total_clients),
        by_vendor: generate_stat_items(vendor_stats, total_clients),
        by_speed: generate_stat_items(speed_stats, total_clients),
        by_status: generate_stat_items(status_stats, total_clients),
    }
}

fn generate_detailed_os_stats(clients: &[Client], _client_hardware_map: &HashMap<String, (Client, Hardware)>) -> OsStats {
    let mut name_stats = HashMap::new();
    let mut version_stats = HashMap::new();
    let mut kernel_stats = HashMap::new();
    let mut arch_stats = HashMap::new();
    
    for client in clients {
        // By name
        let os_name = client.os.as_ref().unwrap_or(&crate::constants::UNKNOWN_VALUE.to_string()).clone();
        name_stats.entry(os_name).or_insert_with(Vec::new).push(client.id.clone());
        
        // By kernel
        let kernel = client.kernel_version.as_ref().unwrap_or(&crate::constants::UNKNOWN_KERNEL.to_string()).clone();
        kernel_stats.entry(kernel).or_insert_with(Vec::new).push(client.id.clone());

        // For now, we'll use placeholder data for version and architecture
        // These would need to be added to the Client model or extracted from hardware info
        version_stats.entry(crate::constants::UNKNOWN_VERSION.to_string()).or_insert_with(Vec::new).push(client.id.clone());
        arch_stats.entry(crate::constants::UNKNOWN_ARCH.to_string()).or_insert_with(Vec::new).push(client.id.clone());
    }
    
    let total_clients = clients.len();
    
    OsStats {
        by_name: generate_stat_items(name_stats, total_clients),
        by_version: generate_stat_items(version_stats, total_clients),
        by_kernel: generate_stat_items(kernel_stats, total_clients),
        by_architecture: generate_stat_items(arch_stats, total_clients),
    }
}

fn generate_detailed_server_stats(clients: &[Client]) -> ServerStats {
    let mut vendor_stats = HashMap::new();
    let mut product_stats = HashMap::new();
    let mut version_stats = HashMap::new();
    
    for client in clients {
        // By vendor
        let vendor = client.sys_vendor.as_ref().unwrap_or(&crate::constants::UNKNOWN_VENDOR.to_string()).clone();
        vendor_stats.entry(vendor).or_insert_with(Vec::new).push(client.id.clone());
        
        // By product name
        let product = client.product_name.as_ref().unwrap_or(&crate::constants::UNKNOWN_MODEL.to_string()).clone();
        product_stats.entry(product).or_insert_with(Vec::new).push(client.id.clone());
        
        // By version (placeholder)
        version_stats.entry(crate::constants::UNKNOWN_VERSION.to_string()).or_insert_with(Vec::new).push(client.id.clone());
    }
    
    let total_clients = clients.len();
    
    ServerStats {
        by_vendor: generate_stat_items(vendor_stats, total_clients),
        by_product_name: generate_stat_items(product_stats, total_clients),
        by_product_version: generate_stat_items(version_stats, total_clients),
    }
}

fn generate_detailed_storage_stats(client_hardware_map: &HashMap<String, (Client, Hardware)>) -> StorageStats {
    let mut type_stats = HashMap::new();
    let mut capacity_stats = HashMap::new();
    let mut vendor_stats = HashMap::new();
    
    for (client, hardware) in client_hardware_map.values() {
        if hardware.disks.is_empty() {
            type_stats.entry(crate::constants::NO_STORAGE.to_string()).or_insert_with(Vec::new).push(client.id.clone());
            capacity_stats.entry("0GB".to_string()).or_insert_with(Vec::new).push(client.id.clone());
            vendor_stats.entry(crate::constants::UNKNOWN_VENDOR.to_string()).or_insert_with(Vec::new).push(client.id.clone());
            continue;
        }
        
        // 更详细的存储类型分类
        let has_nvme = hardware.disks.iter().any(|disk| matches!(disk.storage_type, common::entity::hardware::StorageType::NVMe));
        let has_ssd = hardware.disks.iter().any(|disk| matches!(disk.storage_type, common::entity::hardware::StorageType::SSD));
        let has_hdd = hardware.disks.iter().any(|disk| matches!(disk.storage_type, common::entity::hardware::StorageType::HDD));
        
        // 更精确的存储类型组合统计
        let storage_type_key = match (has_nvme, has_ssd, has_hdd) {
            (true, true, true) => crate::constants::STORAGE_NVME_SSD_HDD,
            (true, true, false) => crate::constants::STORAGE_NVME_SSD,
            (true, false, true) => crate::constants::STORAGE_NVME_HDD,
            (false, true, true) => crate::constants::STORAGE_SSD_HDD,
            (true, false, false) => crate::constants::STORAGE_PURE_NVME,
            (false, true, false) => crate::constants::STORAGE_PURE_SSD,
            (false, false, true) => crate::constants::STORAGE_PURE_HDD,
            (false, false, false) => crate::constants::STORAGE_UNKNOWN_TYPE,
        };
        type_stats.entry(storage_type_key.to_string()).or_insert_with(Vec::new).push(client.id.clone());
        
        // 计算总容量
        let total_capacity: f64 = hardware.disks.iter()
            .filter_map(|d| d.size.parse::<f64>().ok())
            .sum();
        let capacity_key = if total_capacity >= 10000.0 {
            format!("{:.1}TB", total_capacity / 1000.0)
        } else if total_capacity >= 1000.0 {
            format!("{:.1}TB", total_capacity / 1000.0)
        } else {
            format!("{:.0}GB", total_capacity)
        };
        capacity_stats.entry(capacity_key).or_insert_with(Vec::new).push(client.id.clone());
        
        // 主要存储厂商（按容量最大的磁盘）
        if let Some(primary_disk) = hardware.disks.iter()
            .max_by_key(|d| d.size.parse::<f64>().unwrap_or(0.0) as u64) {
            vendor_stats.entry(primary_disk.vendor.clone()).or_insert_with(Vec::new).push(client.id.clone());
        }
    }
    
    let total_clients = client_hardware_map.len();
    
    StorageStats {
        by_type: generate_stat_items(type_stats, total_clients),
        by_capacity: generate_stat_items(capacity_stats, total_clients),
        by_vendor: generate_stat_items(vendor_stats, total_clients),
    }
}

fn generate_stat_items(stats_map: HashMap<String, Vec<String>>, total_clients: usize) -> Vec<StatItem> {
    let mut items: Vec<StatItem> = stats_map.into_iter().map(|(name, client_ids)| {
        let count = client_ids.len();
        let percentage = if total_clients > 0 {
            (count as f64 / total_clients as f64) * 100.0
        } else {
            0.0
        };
        
        StatItem {
            name,
            count,
            percentage,
            client_ids,
        }
    }).collect();

    // Sort by count in descending order
    items.sort_by(|a, b| b.count.cmp(&a.count));
    items
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
            return (StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(response));
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
            os: client.os.clone().unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            kernel_version: client.kernel_version.clone().unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            sys_vendor: client.sys_vendor.clone().unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            product_name: client.product_name.clone().unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
        serial_number: client.serial_number.clone().unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
        last_seen: client.last_seen.clone().unwrap_or_else(|| crate::constants::NEVER_SEEN.to_string()),
        registered_at: client.registered_at.clone().unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            
            // 硬件信息
                    cpu_vendor: hardware.as_ref().map(|h| h.cpu.vendor_id.clone()).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
        cpu_model: hardware.as_ref().map(|h| h.cpu.model_name.clone()).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            cpu_cores: hardware.as_ref().map(|h| h.cpu.cores).unwrap_or(0),
            cpu_threads: hardware.as_ref().map(|h| h.cpu.threads).unwrap_or(0),
            cpu_frequency: hardware.as_ref().map(|h| format!("{:.2} {}", h.cpu.speed as f64 / 1000.0, crate::constants::UNIT_GHZ)).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            
            // 内存信息
                    memory_total: hardware.as_ref().map(|h| format!("{}{}", h.ram.total_size, crate::constants::UNIT_GB)).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
        memory_vendor: hardware.as_ref().map(|h| h.ram.vendor.clone()).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
        memory_speed: hardware.as_ref().map(|h| format!("{}{}", h.ram.speed, crate::constants::UNIT_MHZ)).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            memory_modules: hardware.as_ref().map(|h| h.ram.modules.len() as u32).unwrap_or(0),
            
            // GPU信息
            gpu_count: hardware.as_ref().map(|h| h.gpus.len() as u32).unwrap_or(0),
            gpu_models: hardware.as_ref().map(|h| {
                if h.gpus.is_empty() {
                    crate::constants::UNKNOWN_GPU.to_string()
                                  } else {
                      h.gpus.iter().map(|gpu| gpu.model.clone()).collect::<Vec<_>>().join(", ")
                  }
              }).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
                          gpu_vendors: hardware.as_ref().map(|h| {
                  if h.gpus.is_empty() {
                      crate::constants::COUNT_NONE.to_string()
                } else {
                    let mut vendors: Vec<String> = h.gpus.iter().map(|gpu| gpu.vendor.clone()).collect();
                    vendors.sort();
                    vendors.dedup();
                                          vendors.join(", ")
                  }
              }).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            
            // 存储信息
            storage_count: hardware.as_ref().map(|h| h.disks.len() as u32).unwrap_or(0),
            storage_total: hardware.as_ref().map(|h| {
                let total: f64 = h.disks.iter()
                    .filter_map(|d| d.size.parse::<f64>().ok())
                    .sum();
                if total >= 1000.0 {
                    format!("{:.1}TB", total / 1000.0)
                } else {
                    format!("{:.0}GB", total)
                }
            }).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            storage_types: hardware.as_ref().map(|h| {
                if h.disks.is_empty() {
                    crate::constants::COUNT_NONE.to_string()
                } else {
                    let mut types: Vec<String> = h.disks.iter()
                        .map(|disk| disk.storage_type.to_string())
                        .collect();
                    types.sort();
                    types.dedup();
                    types.join(", ")
                }
            }).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            
            // 网络信息
            network_count: hardware.as_ref().map(|h| h.nics.len() as u32).unwrap_or(0),
            network_types: hardware.as_ref().map(|h| {
                if h.nics.is_empty() {
                    crate::constants::COUNT_NONE.to_string()
                } else {
                    let mut types: Vec<String> = h.nics.iter()
                        .map(|nic| nic.nic_type.to_string())
                        .collect();
                    types.sort();
                    types.dedup();
                    types.join(", ")
                }
            }).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
            network_speeds: hardware.as_ref().map(|h| {
                if h.nics.is_empty() {
                    crate::constants::COUNT_NONE.to_string()
                } else {
                    let speeds: Vec<String> = h.nics.iter()
                        .map(|nic| format!("{}Mbps", nic.speed))
                        .collect();
                    speeds.join(", ")
                }
            }).unwrap_or_else(|| crate::constants::UNKNOWN_VALUE.to_string()),
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