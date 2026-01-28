//! Statistics service for hardware and client data
//!
//! This service provides comprehensive statistics and filtering capabilities
//! for CMDB data including hardware analysis and client reporting.
//! Includes caching for expensive statistical computations.

use crate::cache::{cache_service::key_builder, CacheConfigs, CacheService};
use crate::repository::{client_repository::ClientRepository, hardware_repository::HardwareRepository};
use common::entity::hardware::Hardware;
use common::models::{
    CpuStats, DetailedStats, FilterOptions, GpuStats, MemoryStats,
    NetworkStats, OsStats, ServerStats, StatItem, StorageStats,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Service for generating statistics and analytics
pub struct StatsService {
    client_repo: Arc<ClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
    // Cache for expensive statistical computations
    stats_cache: CacheService<String, serde_json::Value>,
    // Cache for filter options
    filter_options_cache: CacheService<String, FilterOptions>,
}

impl StatsService {
    /// Create a new stats service
    pub fn new(
        client_repo: Arc<ClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
    ) -> Self {
        let cache_configs = CacheConfigs::default();
        Self {
            client_repo,
            hardware_repo,
            stats_cache: CacheService::with_config(cache_configs.stats_data.clone()),
            filter_options_cache: CacheService::with_config(cache_configs.reference_data.clone()),
        }
    }

    /// Create a new stats service with custom cache configs
    pub fn with_cache(
        client_repo: Arc<ClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
        cache_configs: &CacheConfigs,
    ) -> Self {
        Self {
            client_repo,
            hardware_repo,
            stats_cache: CacheService::with_config(cache_configs.stats_data.clone()),
            filter_options_cache: CacheService::with_config(cache_configs.reference_data.clone()),
        }
    }

    /// Get overall statistics with optional category filter
    pub async fn get_overall_stats(
        &self,
        category_filter: Option<&str>,
    ) -> Result<OverallStats, String> {
        // Build cache key
        let cache_key = key_builder::stats(&format!(
            "overall:{}",
            category_filter.unwrap_or("all")
        ));

        // Try cache first
        if let Some(cached) = self.stats_cache.get(&cache_key).await {
            info!("Cache hit for overall stats");
            if let Ok(stats) = serde_json::from_value::<OverallStats>(cached) {
                return Ok(stats);
            }
        }

        info!("Cache miss for overall stats, computing...");

        let clients = self.client_repo.list_all().await.map_err(|e| e.to_string())?;
        let client_hardware_map = self.build_client_hardware_map(&clients).await;

        let total_clients = clients.len();
        let online_clients = self.count_online_clients(&clients);
        let offline_clients = total_clients - online_clients;

        let categories = self.generate_category_stats(&clients, &client_hardware_map, category_filter);

        let result = OverallStats {
            total_clients,
            online_clients,
            offline_clients,
            categories,
        };

        // Cache the result
        if let Ok(json) = serde_json::to_value(&result) {
            self.stats_cache.insert(cache_key, json).await;
        }

        Ok(result)
    }

    /// Get detailed statistics across all categories
    pub async fn get_detailed_stats(&self) -> Result<DetailedStats, String> {
        // Build cache key
        let cache_key = key_builder::stats("detailed");

        // Try cache first
        if let Some(cached) = self.stats_cache.get(&cache_key).await {
            info!("Cache hit for detailed stats");
            if let Ok(stats) = serde_json::from_value::<DetailedStats>(cached) {
                return Ok(stats);
            }
        }

        info!("Cache miss for detailed stats, computing...");

        let clients = self.client_repo.list_all().await.map_err(|e| e.to_string())?;
        let client_hardware_map = self.build_client_hardware_map(&clients).await;

        let total_clients = clients.len();
        let online_clients = self.count_online_clients(&clients);
        let offline_clients = total_clients - online_clients;

        let result = DetailedStats {
            total_clients,
            online_clients,
            offline_clients,
            cpu_stats: self.generate_detailed_cpu_stats(&client_hardware_map),
            memory_stats: self.generate_detailed_memory_stats(&client_hardware_map),
            gpu_stats: self.generate_detailed_gpu_stats(&client_hardware_map),
            network_stats: self.generate_detailed_network_stats(&client_hardware_map),
            os_stats: self.generate_detailed_os_stats(&clients, &client_hardware_map),
            server_stats: self.generate_detailed_server_stats(&clients),
            storage_stats: self.generate_detailed_storage_stats(&client_hardware_map),
        };

        // Cache the result
        if let Ok(json) = serde_json::to_value(&result) {
            self.stats_cache.insert(cache_key, json).await;
        }

        Ok(result)
    }

    /// Get filter options from actual database data
    pub async fn get_filter_options(&self) -> Result<FilterOptions, String> {
        // Build cache key
        let cache_key = key_builder::stats("filter_options");

        // Try cache first
        if let Some(cached) = self.filter_options_cache.get(&cache_key).await {
            info!("Cache hit for filter options");
            return Ok(cached);
        }

        info!("Cache miss for filter options, computing...");

        let clients = self.client_repo.list_all().await.map_err(|e| e.to_string())?;

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
            if let Some(ref os) = client.os
                && !os.trim().is_empty()
            {
                os_names.insert(os.clone());
            }

            if let Some(ref kernel) = client.kernel_version
                && !kernel.trim().is_empty()
            {
                os_kernels.insert(kernel.clone());
            }

            if let Some(ref vendor) = client.sys_vendor
                && !vendor.trim().is_empty()
            {
                server_vendors.insert(vendor.clone());
            }
        }

        // Process hardware data
        for client in &clients {
            if let Ok(Some(hardware)) = self.hardware_repo.get_hardware(&client.id).await {
                if !hardware.cpu.vendor_id.trim().is_empty() {
                    cpu_vendors.insert(hardware.cpu.vendor_id.clone());
                }
                if !hardware.cpu.model_name.trim().is_empty() {
                    cpu_models.insert(hardware.cpu.model_name.clone());
                }

                for gpu in &hardware.gpus {
                    if !gpu.vendor.trim().is_empty() {
                        gpu_vendors.insert(gpu.vendor.clone());
                    }
                    if !gpu.model.trim().is_empty() {
                        gpu_models.insert(gpu.model.clone());
                    }
                }

                for disk in &hardware.disks {
                    let storage_type = disk.storage_type.to_string();
                    if !storage_type.trim().is_empty() {
                        storage_types.insert(storage_type);
                    }
                }

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

        // Sort all vectors
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

        // Cache the result
        self.filter_options_cache.insert(cache_key, filter_options.clone()).await;

        info!("Generated filter options");
        Ok(filter_options)
    }

    // Helper methods

    async fn build_client_hardware_map(&self, clients: &[common::models::Client]) -> HashMap<String, (common::models::Client, Hardware)> {
        let mut map = HashMap::new();
        for client in clients {
            if let Ok(Some(hardware)) = self.hardware_repo.get_hardware(&client.id).await {
                map.insert(client.id.clone(), (client.clone(), hardware));
            }
        }
        map
    }

    fn count_online_clients(&self, clients: &[common::models::Client]) -> usize {
        clients
            .iter()
            .filter(|c| {
                c.last_seen
                    .as_ref()
                    .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                    .map(|dt| {
                        let now = chrono::Utc::now();
                        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                        duration.num_minutes() <= 5
                    })
                    .unwrap_or(false)
            })
            .count()
    }

    fn generate_category_stats(
        &self,
        clients: &[common::models::Client],
        client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>,
        category_filter: Option<&str>,
    ) -> Vec<CategoryStats> {
        let mut categories = Vec::new();

        match category_filter {
            Some("cpu") => {
                categories.push(self.generate_cpu_stats(client_hardware_map));
            }
            Some("memory") => {
                categories.push(self.generate_memory_stats(client_hardware_map));
            }
            Some("gpu") => {
                categories.push(self.generate_gpu_stats(client_hardware_map));
            }
            Some("disk") => {
                categories.push(self.generate_disk_stats(client_hardware_map));
            }
            Some("nic") => {
                categories.push(self.generate_nic_stats(client_hardware_map));
            }
            Some("os") => {
                categories.push(self.generate_os_stats(clients));
            }
            Some("server_model") => {
                categories.push(self.generate_server_model_stats(clients));
            }
            _ => {
                // Generate all categories
                categories.push(self.generate_cpu_stats(client_hardware_map));
                categories.push(self.generate_memory_stats(client_hardware_map));
                categories.push(self.generate_gpu_stats(client_hardware_map));
                categories.push(self.generate_disk_stats(client_hardware_map));
                categories.push(self.generate_nic_stats(client_hardware_map));
                categories.push(self.generate_os_stats(clients));
                categories.push(self.generate_server_model_stats(clients));
            }
        }

        categories
    }

    fn generate_cpu_stats(&self, client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>) -> CategoryStats {
        let mut cpu_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            let cpu_name = format!(
                "{} ({} {})",
                hardware.cpu.model_name,
                hardware.cpu.cores,
                crate::constants::UNIT_CORES
            );
            cpu_stats
                .entry(cpu_name)
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        self.generate_category_stats_internal(
            crate::constants::CATEGORY_CPU_CONFIG,
            cpu_stats,
            client_hardware_map.len(),
        )
    }

    fn generate_memory_stats(&self, client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>) -> CategoryStats {
        let mut memory_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            let memory_name = format!("{}GB", hardware.ram.total_size);
            memory_stats
                .entry(memory_name)
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        self.generate_category_stats_internal(
            crate::constants::CATEGORY_MEMORY_CONFIG,
            memory_stats,
            client_hardware_map.len(),
        )
    }

    fn generate_gpu_stats(&self, client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>) -> CategoryStats {
        let mut gpu_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            if !hardware.gpus.is_empty() {
                let gpu_name = hardware.gpus[0].model.clone();
                gpu_stats
                    .entry(gpu_name)
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
            } else {
                gpu_stats
                    .entry(crate::constants::UNKNOWN_GPU.to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
            }
        }

        self.generate_category_stats_internal(
            crate::constants::CATEGORY_GPU_CONFIG,
            gpu_stats,
            client_hardware_map.len(),
        )
    }

    fn generate_disk_stats(&self, client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>) -> CategoryStats {
        let mut disk_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            if !hardware.disks.is_empty() {
                let total_size: f64 = hardware
                    .disks
                    .iter()
                    .filter_map(|d| d.size.parse::<f64>().ok())
                    .sum();
                let disk_name = format!("{:.0}GB", total_size);
                disk_stats
                    .entry(disk_name)
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
            }
        }

        self.generate_category_stats_internal(
            crate::constants::CATEGORY_STORAGE_CONFIG,
            disk_stats,
            client_hardware_map.len(),
        )
    }

    fn generate_nic_stats(&self, client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>) -> CategoryStats {
        let mut nic_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            let nic_count = hardware.nics.len();
            let nic_name = format!("{} {}", nic_count, crate::constants::COUNT_NICS);
            nic_stats
                .entry(nic_name)
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        self.generate_category_stats_internal(
            crate::constants::CATEGORY_NETWORK_CONFIG,
            nic_stats,
            client_hardware_map.len(),
        )
    }

    fn generate_os_stats(&self, clients: &[common::models::Client]) -> CategoryStats {
        let mut os_stats = HashMap::new();

        for client in clients {
            let os_name = client
                .os
                .as_ref()
                .unwrap_or(&crate::constants::UNKNOWN_SYSTEM.to_string())
                .clone();
            os_stats
                .entry(os_name)
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        self.generate_category_stats_internal(crate::constants::CATEGORY_OS, os_stats, clients.len())
    }

    fn generate_server_model_stats(&self, clients: &[common::models::Client]) -> CategoryStats {
        let mut model_stats = HashMap::new();

        for client in clients {
            let model_name =
                if let (Some(vendor), Some(product)) = (&client.sys_vendor, &client.product_name) {
                    format!("{} {}", vendor, product)
                } else {
                    crate::constants::UNKNOWN_MODEL.to_string()
                };
            model_stats
                .entry(model_name)
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        self.generate_category_stats_internal(
            crate::constants::CATEGORY_SERVER_MODEL,
            model_stats,
            clients.len(),
        )
    }

    fn generate_category_stats_internal(
        &self,
        category_name: &str,
        stats_map: HashMap<String, Vec<String>>,
        total_clients: usize,
    ) -> CategoryStats {
        let items = self.generate_stat_items(stats_map, total_clients);

        CategoryStats {
            category: category_name.to_string(),
            total_clients,
            items,
        }
    }

    fn generate_stat_items(
        &self,
        stats_map: HashMap<String, Vec<String>>,
        total_clients: usize,
    ) -> Vec<StatItem> {
        let mut items: Vec<StatItem> = stats_map
            .into_iter()
            .map(|(name, client_ids)| {
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
            })
            .collect();

        items.sort_by(|a, b| b.count.cmp(&a.count));
        items
    }

    fn generate_detailed_cpu_stats(
        &self,
        client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>,
    ) -> CpuStats {
        let mut vendor_stats = HashMap::new();
        let mut model_stats = HashMap::new();
        let mut cores_stats = HashMap::new();
        let mut threads_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            vendor_stats
                .entry(hardware.cpu.vendor_id.clone())
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            model_stats
                .entry(hardware.cpu.model_name.clone())
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            let cores_key = format!("{} {}", hardware.cpu.cores, crate::constants::UNIT_CORES);
            cores_stats
                .entry(cores_key)
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            let threads_key = format!(
                "{} {}",
                hardware.cpu.threads,
                crate::constants::UNIT_THREADS
            );
            threads_stats
                .entry(threads_key)
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        let total_clients = client_hardware_map.len();

        CpuStats {
            by_vendor: self.generate_stat_items(vendor_stats, total_clients),
            by_model: self.generate_stat_items(model_stats, total_clients),
            by_cores: self.generate_stat_items(cores_stats, total_clients),
            by_threads: self.generate_stat_items(threads_stats, total_clients),
        }
    }

    fn generate_detailed_memory_stats(
        &self,
        client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>,
    ) -> MemoryStats {
        let mut capacity_stats = HashMap::new();
        let mut vendor_stats = HashMap::new();
        let mut type_stats = HashMap::new();
        let mut speed_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            let capacity_key = format!("{}GB", hardware.ram.total_size);
            capacity_stats
                .entry(capacity_key)
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            vendor_stats
                .entry(hardware.ram.vendor.clone())
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            if !hardware.ram.modules.is_empty() {
                let memory_type = hardware.ram.modules[0].memory_type.clone();
                type_stats
                    .entry(memory_type)
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
            }

            let speed_key = format!("{}MHz", hardware.ram.speed);
            speed_stats
                .entry(speed_key)
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        let total_clients = client_hardware_map.len();

        MemoryStats {
            by_capacity: self.generate_stat_items(capacity_stats, total_clients),
            by_vendor: self.generate_stat_items(vendor_stats, total_clients),
            by_type: self.generate_stat_items(type_stats, total_clients),
            by_speed: self.generate_stat_items(speed_stats, total_clients),
        }
    }

    fn generate_detailed_gpu_stats(
        &self,
        client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>,
    ) -> GpuStats {
        let mut vendor_stats = HashMap::new();
        let mut model_stats = HashMap::new();
        let mut model_with_count_stats = HashMap::new();
        let mut driver_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            if hardware.gpus.is_empty() {
                vendor_stats
                    .entry(crate::constants::UNKNOWN_GPU.to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
                model_stats
                    .entry(crate::constants::UNKNOWN_GPU.to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
                model_with_count_stats
                    .entry(crate::constants::UNKNOWN_GPU.to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
                driver_stats
                    .entry(crate::constants::UNKNOWN_DRIVER.to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
            } else {
                let primary_gpu = &hardware.gpus[0];
                vendor_stats
                    .entry(primary_gpu.vendor.clone())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
                driver_stats
                    .entry(primary_gpu.driver_version.clone())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());

                let mut unique_models = std::collections::HashSet::new();
                for gpu in &hardware.gpus {
                    unique_models.insert(gpu.model.clone());
                }
                for model in unique_models {
                    model_stats
                        .entry(model)
                        .or_insert_with(Vec::new)
                        .push(client.id.clone());
                }

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
                    model_with_count_stats
                        .entry(model_with_count_key)
                        .or_insert_with(Vec::new)
                        .push(client.id.clone());
                }
            }
        }

        let total_clients = client_hardware_map.len();

        GpuStats {
            by_vendor: self.generate_stat_items(vendor_stats, total_clients),
            by_model: self.generate_stat_items(model_stats, total_clients),
            by_model_with_count: self.generate_stat_items(model_with_count_stats, total_clients),
            by_driver_version: self.generate_stat_items(driver_stats, total_clients),
        }
    }

    fn generate_detailed_network_stats(
        &self,
        client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>,
    ) -> NetworkStats {
        let mut type_stats = HashMap::new();
        let mut vendor_stats = HashMap::new();
        let mut speed_stats = HashMap::new();
        let mut status_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            if !hardware.nics.is_empty() {
                let primary_nic = &hardware.nics[0];

                let type_key = primary_nic.nic_type.to_string();
                type_stats
                    .entry(type_key)
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());

                vendor_stats
                    .entry(primary_nic.vendor.clone())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());

                let max_speed = hardware.nics.iter().map(|nic| nic.speed).max().unwrap_or(0);
                let speed_key = format!("{}Mbps", max_speed);
                speed_stats
                    .entry(speed_key)
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());

                let has_up_nic = hardware
                    .nics
                    .iter()
                    .any(|nic| matches!(nic.status, common::entity::hardware::NICStatus::Up));
                let status_key = if has_up_nic {
                    "Up".to_string()
                } else {
                    "Down".to_string()
                };
                status_stats
                    .entry(status_key)
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
            }
        }

        let total_clients = client_hardware_map.len();

        NetworkStats {
            by_type: self.generate_stat_items(type_stats, total_clients),
            by_vendor: self.generate_stat_items(vendor_stats, total_clients),
            by_speed: self.generate_stat_items(speed_stats, total_clients),
            by_status: self.generate_stat_items(status_stats, total_clients),
        }
    }

    fn generate_detailed_os_stats(
        &self,
        clients: &[common::models::Client],
        _client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>,
    ) -> OsStats {
        let mut name_stats = HashMap::new();
        let mut version_stats = HashMap::new();
        let mut kernel_stats = HashMap::new();
        let mut arch_stats = HashMap::new();

        for client in clients {
            let os_name = client
                .os
                .as_ref()
                .unwrap_or(&crate::constants::UNKNOWN_VALUE.to_string())
                .clone();
            name_stats
                .entry(os_name)
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            let kernel = client
                .kernel_version
                .as_ref()
                .unwrap_or(&crate::constants::UNKNOWN_KERNEL.to_string())
                .clone();
            kernel_stats
                .entry(kernel)
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            version_stats
                .entry(crate::constants::UNKNOWN_VERSION.to_string())
                .or_insert_with(Vec::new)
                .push(client.id.clone());
            arch_stats
                .entry(crate::constants::UNKNOWN_ARCH.to_string())
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        let total_clients = clients.len();

        OsStats {
            by_name: self.generate_stat_items(name_stats, total_clients),
            by_version: self.generate_stat_items(version_stats, total_clients),
            by_kernel: self.generate_stat_items(kernel_stats, total_clients),
            by_architecture: self.generate_stat_items(arch_stats, total_clients),
        }
    }

    fn generate_detailed_server_stats(&self, clients: &[common::models::Client]) -> ServerStats {
        let mut vendor_stats = HashMap::new();
        let mut product_stats = HashMap::new();
        let mut version_stats = HashMap::new();

        for client in clients {
            let vendor = client
                .sys_vendor
                .as_ref()
                .unwrap_or(&crate::constants::UNKNOWN_VENDOR.to_string())
                .clone();
            vendor_stats
                .entry(vendor)
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            let product = client
                .product_name
                .as_ref()
                .unwrap_or(&crate::constants::UNKNOWN_MODEL.to_string())
                .clone();
            product_stats
                .entry(product)
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            version_stats
                .entry(crate::constants::UNKNOWN_VERSION.to_string())
                .or_insert_with(Vec::new)
                .push(client.id.clone());
        }

        let total_clients = clients.len();

        ServerStats {
            by_vendor: self.generate_stat_items(vendor_stats, total_clients),
            by_product_name: self.generate_stat_items(product_stats, total_clients),
            by_product_version: self.generate_stat_items(version_stats, total_clients),
        }
    }

    fn generate_detailed_storage_stats(
        &self,
        client_hardware_map: &HashMap<String, (common::models::Client, Hardware)>,
    ) -> StorageStats {
        let mut type_stats = HashMap::new();
        let mut capacity_stats = HashMap::new();
        let mut vendor_stats = HashMap::new();

        for (client, hardware) in client_hardware_map.values() {
            if hardware.disks.is_empty() {
                type_stats
                    .entry(crate::constants::NO_STORAGE.to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
                capacity_stats
                    .entry("0GB".to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
                vendor_stats
                    .entry(crate::constants::UNKNOWN_VENDOR.to_string())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
                continue;
            }

            let has_nvme = hardware.disks.iter().any(|disk| {
                matches!(
                    disk.storage_type,
                    common::entity::hardware::StorageType::NVMe
                )
            });
            let has_ssd = hardware.disks.iter().any(|disk| {
                matches!(
                    disk.storage_type,
                    common::entity::hardware::StorageType::SSD
                )
            });
            let has_hdd = hardware.disks.iter().any(|disk| {
                matches!(
                    disk.storage_type,
                    common::entity::hardware::StorageType::HDD
                )
            });

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
            type_stats
                .entry(storage_type_key.to_string())
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            let total_capacity: f64 = hardware
                .disks
                .iter()
                .filter_map(|d| d.size.parse::<f64>().ok())
                .sum();
            let capacity_key = if total_capacity >= 10000.0 {
                format!("{:.1}TB", total_capacity / 1000.0)
            } else if total_capacity >= 1000.0 {
                format!("{:.1}TB", total_capacity / 1000.0)
            } else {
                format!("{:.0}GB", total_capacity)
            };
            capacity_stats
                .entry(capacity_key)
                .or_insert_with(Vec::new)
                .push(client.id.clone());

            if let Some(primary_disk) = hardware
                .disks
                .iter()
                .max_by_key(|d| d.size.parse::<f64>().unwrap_or(0.0) as u64)
            {
                vendor_stats
                    .entry(primary_disk.vendor.clone())
                    .or_insert_with(Vec::new)
                    .push(client.id.clone());
            }
        }

        let total_clients = client_hardware_map.len();

        StorageStats {
            by_type: self.generate_stat_items(type_stats, total_clients),
            by_capacity: self.generate_stat_items(capacity_stats, total_clients),
            by_vendor: self.generate_stat_items(vendor_stats, total_clients),
        }
    }
}

/// Category statistics for a specific hardware type
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub total_clients: usize,
    pub items: Vec<StatItem>,
}

/// Overall statistics summary
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OverallStats {
    pub total_clients: usize,
    pub online_clients: usize,
    pub offline_clients: usize,
    pub categories: Vec<CategoryStats>,
}
