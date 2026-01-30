//! Client filter service for advanced filtering and searching
//!
//! This service provides filtering and search capabilities for clients,
//! including hardware-based filtering and search functionality.

use crate::repository::{client_repository::ClientRepository, hardware_repository::HardwareRepository};
use common::entity::hardware::Hardware;
use common::models::{Client, FilterOptions};
use serde::Deserialize;
use std::sync::Arc;

/// Query parameters for basic client search
#[derive(Debug, Deserialize, Clone)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub os: Option<String>,
    pub status: Option<String>,
}

/// Query parameters for hardware filtering
#[derive(Debug, Deserialize, Clone)]
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

/// Service for filtering and searching clients
pub struct ClientFilterService {
    client_repo: Arc<ClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
}

impl ClientFilterService {
    /// Create a new client filter service
    pub fn new(
        client_repo: Arc<ClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
    ) -> Self {
        Self {
            client_repo,
            hardware_repo,
        }
    }

    /// Search clients with basic filters
    pub async fn search_clients(&self, query: &SearchQuery) -> Result<Vec<Client>, String> {
        let clients = self.client_repo.list_all().await.map_err(|e| e.to_string())?;

        let mut filtered_clients = Vec::new();

        for client in clients {
            let hardware = self
                .hardware_repo
                .get_hardware(&client.id)
                .await
                .unwrap_or(None);

            // Check search term
            let matches_search = if let Some(ref search_term) = query.q {
                if search_term.is_empty() {
                    true
                } else {
                    self.client_matches_search(&client, hardware.as_ref(), search_term)
                }
            } else {
                true
            };

            // Check OS filter
            let matches_os = if let Some(ref os_filter) = query.os {
                if os_filter == "all" {
                    true
                } else {
                    client.os.as_ref() == Some(os_filter)
                }
            } else {
                true
            };

            // Check status filter
            let matches_status = if let Some(ref status_filter) = query.status {
                if status_filter == "all" {
                    true
                } else {
                    let is_online = self.is_client_online(&client);
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
                filtered_clients.push(client);
            }
        }

        Ok(filtered_clients)
    }

    /// Filter clients by hardware specifications
    pub async fn filter_clients_by_hardware(
        &self,
        params: &HardwareFilterQuery,
    ) -> Result<Vec<Client>, String> {
        let clients = self.client_repo.list_all().await.map_err(|e| e.to_string())?;

        let mut filtered_clients = Vec::new();

        for client in clients {
            let hardware = self
                .hardware_repo
                .get_hardware(&client.id)
                .await
                .unwrap_or(None);

            if self.client_matches_hardware_filters(&client, hardware.as_ref(), params) {
                filtered_clients.push(client);
            }
        }

        Ok(filtered_clients)
    }

    /// Get filter options based on client IDs
    pub async fn get_filter_options_by_client_ids(
        &self,
        client_ids: &[String],
    ) -> Result<FilterOptions, String> {
        if client_ids.is_empty() {
            return Ok(FilterOptions::default());
        }

        let mut clients = Vec::new();
        for client_id in client_ids {
            match self.client_repo.get(client_id).await {
                Ok(Some(client)) => clients.push(client),
                Ok(None) => continue,
                Err(_) => continue,
            }
        }

        if clients.is_empty() {
            return Ok(FilterOptions::default());
        }

        let mut hardware_list = Vec::new();
        for client in &clients {
            match self.hardware_repo.get_hardware(&client.id).await {
                Ok(Some(hardware)) => hardware_list.push(hardware),
                Ok(None) => continue,
                Err(_) => continue,
            }
        }

        Ok(self.generate_filter_options(&hardware_list))
    }

    // Helper methods

    fn is_client_online(&self, client: &Client) -> bool {
        client
            .last_seen
            .as_ref()
            .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
            .map(|dt| {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                duration.num_minutes() <= 5
            })
            .unwrap_or(false)
    }

    fn client_matches_search(
        &self,
        client: &Client,
        hardware: Option<&Hardware>,
        search_term: &str,
    ) -> bool {
        let search_lower = search_term.to_lowercase();

        // Search client basic info
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

        // Search hardware info
        let hardware_matches = if let Some(hw) = hardware {
            let os_matches = hw.cpu.vendor_id.to_lowercase().contains(&search_lower)
                || hw.cpu.model_name.to_lowercase().contains(&search_lower);

            let gpu_matches = hw.gpus.iter().any(|gpu| {
                gpu.vendor.to_lowercase().contains(&search_lower)
                    || gpu.model.to_lowercase().contains(&search_lower)
                    || gpu.device_id.to_lowercase().contains(&search_lower)
                    || gpu.driver_version.to_lowercase().contains(&search_lower)
            });

            let ram_matches = hw.ram.vendor.to_lowercase().contains(&search_lower)
                || hw.ram.model.to_lowercase().contains(&search_lower)
                || hw.ram.form_factor.to_lowercase().contains(&search_lower)
                || hw.ram.modules.iter().any(|module| {
                    module.vendor.to_lowercase().contains(&search_lower)
                        || module.part_number.to_lowercase().contains(&search_lower)
                        || module.memory_type.to_lowercase().contains(&search_lower)
                });

            let disk_matches = hw.disks.iter().any(|disk| {
                disk.vendor.to_lowercase().contains(&search_lower)
                    || disk.model.to_lowercase().contains(&search_lower)
                    || disk.serial_number.to_lowercase().contains(&search_lower)
                    || disk.firmware_version.to_lowercase().contains(&search_lower)
            });

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

    fn client_matches_hardware_filters(
        &self,
        client: &Client,
        hardware: Option<&Hardware>,
        params: &HardwareFilterQuery,
    ) -> bool {
        // Apply search term filter
        let matches_search = if let Some(ref search_term) = params.search_term {
            if search_term.is_empty() {
                true
            } else {
                self.client_matches_search(client, hardware, search_term)
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
        let matches_server_vendor = if let Some(ref vendor_filter) = params.server_vendor_filter {
            if vendor_filter.is_empty() || vendor_filter == crate::constants::FILTER_ALL {
                true
            } else {
                client.sys_vendor.as_ref() == Some(vendor_filter)
            }
        } else {
            true
        };

        // Apply server model filter
        let matches_server_model = if let Some(ref model_filter) = params.server_model_filter {
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
                let is_online = self.is_client_online(client);
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
        let matches_client_status = if let Some(ref status_filter) = params.client_status_filter {
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
        let matches_hardware = if let Some(hw) = hardware {
            self.hardware_matches_filters(hw, params)
        } else {
            // If no hardware data available, only exclude if hardware filters are set
            let has_hardware_filters = params.cpu_vendor_filter.is_some()
                || params.cpu_model_filter.is_some()
                || params.gpu_vendor_filter.is_some()
                || params.gpu_model_filter.is_some()
                || params.memory_min_filter.is_some()
                || params.memory_max_filter.is_some()
                || params.network_type_filter.is_some()
                || params.network_model_filter.is_some()
                || params.storage_type_filter.is_some();

            !has_hardware_filters
        };

        matches_search
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
    }

    fn hardware_matches_filters(&self, hw: &Hardware, params: &HardwareFilterQuery) -> bool {
        // CPU vendor filter
        let matches_cpu_vendor = if let Some(ref cpu_vendor_filter) = params.cpu_vendor_filter {
            if cpu_vendor_filter.is_empty() || cpu_vendor_filter == crate::constants::FILTER_ALL {
                true
            } else {
                hw.cpu.vendor_id == *cpu_vendor_filter
            }
        } else {
            true
        };

        // CPU model filter
        let matches_cpu_model = if let Some(ref cpu_model_filter) = params.cpu_model_filter {
            if cpu_model_filter.is_empty() || cpu_model_filter == crate::constants::FILTER_ALL {
                true
            } else {
                hw.cpu.model_name == *cpu_model_filter
            }
        } else {
            true
        };

        // GPU vendor filter
        let matches_gpu_vendor = if let Some(ref gpu_vendor_filter) = params.gpu_vendor_filter {
            if gpu_vendor_filter.is_empty() || gpu_vendor_filter == crate::constants::FILTER_ALL {
                true
            } else {
                hw.gpus.iter().any(|gpu| gpu.vendor == *gpu_vendor_filter)
            }
        } else {
            true
        };

        // GPU model filter
        let matches_gpu_model = if let Some(ref gpu_model_filter) = params.gpu_model_filter {
            if gpu_model_filter.is_empty() || gpu_model_filter == crate::constants::FILTER_ALL {
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
        let matches_network_type = if let Some(ref network_type_filter) = params.network_type_filter {
            if network_type_filter.is_empty() || network_type_filter == crate::constants::FILTER_ALL {
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
        let matches_network_model = if let Some(ref network_model_filter) = params.network_model_filter {
            if network_model_filter.is_empty() || network_model_filter == crate::constants::FILTER_ALL {
                true
            } else {
                hw.nics.iter().any(|nic| nic.model == *network_model_filter)
            }
        } else {
            true
        };

        // Storage type filter
        let matches_storage = if let Some(ref storage_type_filter) = params.storage_type_filter {
            if storage_type_filter.is_empty() || storage_type_filter == crate::constants::FILTER_ALL {
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
    }

    /// Get hardware export data for a client
    pub async fn get_hardware_export_data(
        &self,
        client_id: &str,
    ) -> Result<Option<common::models::ClientHardwareExport>, String> {
        let client = match self.client_repo.get(client_id).await {
            Ok(Some(client)) => client,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err.to_string()),
        };

        let hardware = self
            .hardware_repo
            .get_hardware(client_id)
            .await
            .unwrap_or(None);

        let export_data = common::models::ClientHardwareExport {
            client_id: client.id.clone(),
            hostname: client.hostname.clone(),
            ip_address: client.ip_address.clone(),
            os: client.os.clone().unwrap_or_default(),
            kernel_version: client.kernel_version.clone().unwrap_or_default(),
            sys_vendor: client.sys_vendor.clone().unwrap_or_default(),
            product_name: client.product_name.clone().unwrap_or_default(),
            serial_number: client.serial_number.clone().unwrap_or_default(),
            last_seen: client.last_seen.clone().unwrap_or_default(),
            registered_at: client.registered_at.clone().unwrap_or_default(),
            cpu_vendor: hardware.as_ref().map(|h| h.cpu.vendor_id.clone()).unwrap_or_default(),
            cpu_model: hardware.as_ref().map(|h| h.cpu.model_name.clone()).unwrap_or_default(),
            cpu_cores: hardware.as_ref().map(|h| h.cpu.cores).unwrap_or(0),
            cpu_threads: hardware.as_ref().map(|h| h.cpu.threads).unwrap_or(0),
            cpu_frequency: hardware.as_ref().map(|h| format!("{} MHz", h.cpu.speed)).unwrap_or_default(),
            memory_total: hardware.as_ref().map(|h| format!("{} GB", h.ram.total_size)).unwrap_or_default(),
            memory_vendor: hardware.as_ref().map(|h| h.ram.vendor.clone()).unwrap_or_default(),
            memory_speed: hardware.as_ref().map(|h| format!("{} MHz", h.ram.speed)).unwrap_or_default(),
            memory_modules: hardware.as_ref().map(|h| h.ram.count as u32).unwrap_or(0),
            gpu_count: hardware.as_ref().map(|h| h.gpus.len() as u32).unwrap_or(0),
            gpu_models: hardware.as_ref().map(|h| {
                h.gpus.iter().map(|g| g.model.clone()).collect::<Vec<_>>().join(", ")
            }).unwrap_or_default(),
            gpu_vendors: hardware.as_ref().map(|h| {
                h.gpus.iter().map(|g| g.vendor.clone()).collect::<Vec<_>>().join(", ")
            }).unwrap_or_default(),
            storage_count: hardware.as_ref().map(|h| h.disks.len() as u32).unwrap_or(0),
            storage_total: hardware.as_ref().map(|h| {
                h.disks.iter().map(|d| format!("{} {}", d.size, d.size_unit)).collect::<Vec<_>>().join(", ")
            }).unwrap_or_default(),
            storage_types: hardware.as_ref().map(|h| {
                h.disks.iter().map(|d| d.storage_type.to_string()).collect::<Vec<_>>().join(", ")
            }).unwrap_or_default(),
            network_count: hardware.as_ref().map(|h| h.nics.len() as u32).unwrap_or(0),
            network_types: hardware.as_ref().map(|h| {
                h.nics.iter().map(|n| n.nic_type.to_string()).collect::<Vec<_>>().join(", ")
            }).unwrap_or_default(),
            network_speeds: hardware.as_ref().map(|h| {
                h.nics.iter().map(|n| format!("{} Mbps", n.speed)).collect::<Vec<_>>().join(", ")
            }).unwrap_or_default(),
        };

        Ok(Some(export_data))
    }

    fn generate_filter_options(&self, hardware_list: &[Hardware]) -> FilterOptions {
        use std::collections::HashSet;

        let mut cpu_vendors = HashSet::new();
        let mut cpu_models = HashSet::new();
        let mut gpu_vendors = HashSet::new();
        let mut gpu_models = HashSet::new();
        let mut network_types = HashSet::new();
        let mut network_models = HashSet::new();
        let mut storage_types = HashSet::new();

        for hardware in hardware_list {
            // CPU info
            if !hardware.cpu.vendor_id.is_empty() {
                cpu_vendors.insert(hardware.cpu.vendor_id.clone());
            }
            if !hardware.cpu.model_name.is_empty() {
                cpu_models.insert(hardware.cpu.model_name.clone());
            }

            // GPU info
            for gpu in &hardware.gpus {
                if !gpu.vendor.is_empty() {
                    gpu_vendors.insert(gpu.vendor.clone());
                }
                if !gpu.model.is_empty() {
                    gpu_models.insert(gpu.model.clone());
                }
            }

            // Network info
            for nic in &hardware.nics {
                network_types.insert(nic.nic_type.to_string());
                if !nic.model.is_empty() {
                    network_models.insert(nic.model.clone());
                }
            }

            // Storage info
            for disk in &hardware.disks {
                storage_types.insert(disk.storage_type.to_string());
            }
        }

        FilterOptions {
            cpu_vendors: {
                let mut vec: Vec<String> = cpu_vendors.into_iter().collect();
                vec.sort();
                vec
            },
            cpu_models: {
                let mut vec: Vec<String> = cpu_models.into_iter().collect();
                vec.sort();
                vec
            },
            gpu_vendors: {
                let mut vec: Vec<String> = gpu_vendors.into_iter().collect();
                vec.sort();
                vec
            },
            gpu_models: {
                let mut vec: Vec<String> = gpu_models.into_iter().collect();
                vec.sort();
                vec
            },
            network_types: {
                let mut vec: Vec<String> = network_types.into_iter().collect();
                vec.sort();
                vec
            },
            network_models: {
                let mut vec: Vec<String> = network_models.into_iter().collect();
                vec.sort();
                vec
            },
            storage_types: {
                let mut vec: Vec<String> = storage_types.into_iter().collect();
                vec.sort();
                vec
            },
            os_names: vec![],
            os_kernels: vec![],
            server_vendors: vec![],
        }
    }
}
