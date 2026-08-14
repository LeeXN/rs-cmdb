//! Export service for client hardware data
//!
//! This service handles the transformation of client and hardware data
//! into export-friendly formats.

use crate::repository::{
    client_repository::ClientRepository, hardware_repository::HardwareRepository,
};
use common::models::ClientHardwareExport;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

pub struct ExportService {
    client_repo: Arc<ClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
}

impl ExportService {
    pub fn new(client_repo: Arc<ClientRepository>, hardware_repo: Arc<HardwareRepository>) -> Self {
        Self {
            client_repo,
            hardware_repo,
        }
    }

    #[allow(dead_code)]
    pub async fn export_client_hardware_data(&self) -> Result<Vec<ClientHardwareExport>, String> {
        self.export_client_hardware_data_for_client_ids(None).await
    }

    /// Export hardware data restricted to the supplied client IDs.
    pub async fn export_client_hardware_data_for_client_ids(
        &self,
        allowed_client_ids: Option<&HashSet<String>>,
    ) -> Result<Vec<ClientHardwareExport>, String> {
        // Get all clients
        let mut clients = self
            .client_repo
            .list_all()
            .await
            .map_err(|e| e.to_string())?;
        if let Some(allowed_client_ids) = allowed_client_ids {
            clients.retain(|client| allowed_client_ids.contains(&client.id));
        }

        let mut export_data = Vec::new();

        for client in clients {
            let hardware = self
                .hardware_repo
                .get_hardware(&client.id)
                .await
                .unwrap_or(None);

            let export_item = ClientHardwareExport {
                // 基本信息
                client_id: client.id.clone(),
                hostname: client.hostname.clone(),
                ip_address: client.ip_address.clone(),
                primary_ip: client.primary_ip.clone(),
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
        Ok(export_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::tests::fixtures::{create_test_hardware_info, setup_test_db};
    use common::models::Client;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_export_empty() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(db.clone()));
        let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));
        let svc = ExportService::new(client_repo, hardware_repo);

        let data = svc.export_client_hardware_data().await.unwrap();
        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn test_export_with_data() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(db.clone()));
        let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));

        let client = Client::new("export-test.example.com".into(), "10.0.0.4".into());
        client_repo.save(&client).await.unwrap();

        let hw = create_test_hardware_info(&client.id);
        hardware_repo
            .save_hardware(&client.id, &hw, false)
            .await
            .unwrap();

        let svc = ExportService::new(client_repo, hardware_repo);
        let data = svc.export_client_hardware_data().await.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].hostname, "export-test.example.com");
    }
}
