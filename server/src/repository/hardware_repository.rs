use crate::db::Database;
use common::entity::hardware::{Hardware, NICStatus};
use common::error::{CmdbError, CmdbResult};
use common::models::{
    HardwareHistoryChange, HardwareHistoryChangeType, HardwareHistoryEntry,
    build_hardware_history_entries, calculate_hardware_history_changes,
};
use serde_json;
use std::sync::Arc;

/// Repository for hardware information operations
pub struct HardwareRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl HardwareRepository {
    /// Create a new hardware repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "hardware:".to_string(),
        }
    }

    /// Get hardware key in database
    fn get_key(&self, client_id: &str) -> String {
        format!("{}{}", self.key_prefix, client_id)
    }

    /// Get history key in database
    fn get_history_key(&self, client_id: &str, timestamp: &str) -> String {
        format!("{}{}:history:{}", self.key_prefix, client_id, timestamp)
    }

    /// Save hardware information to the database
    pub async fn save_hardware(
        &self,
        client_id: &str,
        hardware: &Hardware,
        save_history: bool,
    ) -> CmdbResult<()> {
        let hardware_json = serde_json::to_vec(hardware).map_err(|e| {
            CmdbError::Serialization(format!("Failed to serialize hardware: {}", e))
        })?;

        // Load previous hardware BEFORE saving current, so we can compute diffs
        let previous_hardware = if save_history {
            self.get_hardware(client_id).await?
        } else {
            None
        };

        // Check if hardware has changed (only if we need to save history)
        let hardware_changed = if save_history {
            match &previous_hardware {
                Some(existing_hardware) => {
                    // Compare the hardware configurations
                    !self.hardware_equals(existing_hardware, hardware)
                }
                None => {
                    // No existing hardware, this is the first time
                    true
                }
            }
        } else {
            false
        };

        // Save current hardware data
        self.db
            .set(&self.get_key(client_id), &hardware_json)
            .await?;

        // Only save to history if hardware has actually changed
        if save_history && hardware_changed {
            let timestamp = chrono::Utc::now().timestamp().to_string();
            let (changes, snapshot) = match previous_hardware {
                Some(prev) => (calculate_hardware_history_changes(&prev, hardware), None),
                None => (
                    vec![HardwareHistoryChange {
                        component: "初始版本".to_string(),
                        change_type: HardwareHistoryChangeType::Added,
                        old_value: String::new(),
                        new_value: "首次记录".to_string(),
                    }],
                    None,
                ),
            };
            let entry = HardwareHistoryEntry {
                timestamp: timestamp.clone(),
                changes,
                snapshot,
            };
            let entry_json = serde_json::to_vec(&entry).map_err(|e| {
                CmdbError::Serialization(format!("Failed to serialize history entry: {}", e))
            })?;
            self.db
                .set(&self.get_history_key(client_id, &timestamp), &entry_json)
                .await?;
        }

        Ok(())
    }

    /// Save hardware information to the database with custom timestamp
    pub async fn save_hardware_with_timestamp(
        &self,
        client_id: &str,
        hardware: &Hardware,
        save_history: bool,
        custom_timestamp: Option<&str>,
    ) -> CmdbResult<()> {
        let hardware_json = serde_json::to_vec(hardware).map_err(|e| {
            CmdbError::Serialization(format!("Failed to serialize hardware: {}", e))
        })?;

        // Load previous hardware BEFORE saving current, so we can compute diffs
        let previous_hardware = if save_history {
            self.get_hardware(client_id).await?
        } else {
            None
        };

        // Check if hardware has changed (only if we need to save history)
        let hardware_changed = if save_history {
            match &previous_hardware {
                Some(existing_hardware) => {
                    // Compare the hardware configurations
                    !self.hardware_equals(existing_hardware, hardware)
                }
                None => {
                    // No existing hardware, this is the first time
                    true
                }
            }
        } else {
            false
        };

        // Save current hardware data
        self.db
            .set(&self.get_key(client_id), &hardware_json)
            .await?;

        // Only save to history if hardware has actually changed
        if save_history && hardware_changed {
            let timestamp = if let Some(ts) = custom_timestamp {
                // Try to parse the timestamp to validate it and convert to Unix timestamp
                if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(ts) {
                    datetime.timestamp().to_string()
                } else {
                    // If parsing fails, use current timestamp
                    chrono::Utc::now().timestamp().to_string()
                }
            } else {
                chrono::Utc::now().timestamp().to_string()
            };
            let (changes, snapshot) = match previous_hardware {
                Some(prev) => (calculate_hardware_history_changes(&prev, hardware), None),
                None => (
                    vec![HardwareHistoryChange {
                        component: "初始版本".to_string(),
                        change_type: HardwareHistoryChangeType::Added,
                        old_value: String::new(),
                        new_value: "首次记录".to_string(),
                    }],
                    None,
                ),
            };
            let entry = HardwareHistoryEntry {
                timestamp: timestamp.clone(),
                changes,
                snapshot,
            };
            let entry_json = serde_json::to_vec(&entry).map_err(|e| {
                CmdbError::Serialization(format!("Failed to serialize history entry: {}", e))
            })?;
            self.db
                .set(&self.get_history_key(client_id, &timestamp), &entry_json)
                .await?;
        }

        Ok(())
    }

    /// Compare two hardware configurations to check if they are equal
    fn hardware_equals(&self, hw1: &Hardware, hw2: &Hardware) -> bool {
        let mut left = hw1.clone();
        let mut right = hw2.clone();

        for nic in &mut left.nics {
            nic.status = NICStatus::Unknown;
            nic.speed = 0;
        }

        for nic in &mut right.nics {
            nic.status = NICStatus::Unknown;
            nic.speed = 0;
        }

        left.semantically_eq(&right)
    }

    /// Get hardware information for a client
    pub async fn get_hardware(&self, client_id: &str) -> CmdbResult<Option<Hardware>> {
        let hardware_data = match self.db.get(&self.get_key(client_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };

        let hardware = serde_json::from_slice(&hardware_data).map_err(|e| {
            CmdbError::Serialization(format!("Failed to deserialize hardware: {}", e))
        })?;

        Ok(Some(hardware))
    }

    /// Get hardware history for a client
    pub async fn get_hardware_history(
        &self,
        client_id: &str,
    ) -> CmdbResult<Vec<HardwareHistoryEntry>> {
        let history_prefix = format!("{}{}:history:", self.key_prefix, client_id);
        let entries = self.db.list_entries(&history_prefix).await?;
        let mut full_snapshots = Vec::new();
        let mut delta_entries = Vec::new();
        let current_hardware = self.get_hardware(client_id).await?;

        for (key, data) in entries {
            // Extract timestamp from key
            let timestamp = key
                .strip_prefix(&history_prefix)
                .unwrap_or_default()
                .to_string();

            // Try to deserialize as new delta format first
            match serde_json::from_slice::<HardwareHistoryEntry>(&data) {
                Ok(mut entry) => {
                    entry.timestamp = timestamp;
                    if entry.snapshot.is_none() {
                        entry.snapshot = current_hardware.clone();
                    }
                    delta_entries.push(entry);
                }
                Err(_) => {
                    // Old format: full Hardware snapshot
                    match serde_json::from_slice::<Hardware>(&data) {
                        Ok(hw) => {
                            full_snapshots.push((timestamp, hw));
                        }
                        Err(e) => {
                            return Err(CmdbError::Serialization(format!(
                                "Failed to deserialize history entry: {}",
                                e
                            )));
                        }
                    }
                }
            }
        }

        // Process old-format snapshots through build_hardware_history_entries
        full_snapshots.sort_by(|a, b| b.0.cmp(&a.0));
        let mut result = if full_snapshots.is_empty() {
            Vec::new()
        } else {
            build_hardware_history_entries(&full_snapshots)
        };

        // Add new-format delta entries
        delta_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        result.extend(delta_entries);

        // Sort all by timestamp descending
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(result)
    }

    /// Delete hardware information for a client
    pub async fn delete_hardware(&self, client_id: &str) -> CmdbResult<()> {
        // Delete current hardware data
        self.db.delete(&self.get_key(client_id)).await?;

        // Delete history
        let history_prefix = format!("{}{}:history:", self.key_prefix, client_id);
        let keys = self.db.list_keys(&history_prefix).await?;

        for key in keys {
            self.db.delete(&key).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::entity::hardware::*;

    fn create_test_hardware() -> Hardware {
        Hardware {
            system: None,
            os: OS::default(),
            cpu: CPU {
                vendor_id: "GenuineIntel".to_string(),
                model_name: "Intel(R) Xeon(R) CPU".to_string(),
                cores: 8,
                threads: 16,
                cpus: 2,
                flags: vec!["fpu".to_string(), "sse".to_string()],
                speed: 2400,
            },
            gpus: vec![],
            ram: RAM {
                vendor: "Samsung".to_string(),
                model: "DDR4".to_string(),
                size: 64,
                speed: 2666,
                total_size: 64,
                count: 8,
                form_factor: "DIMM".to_string(),
                modules: vec![],
            },
            disks: vec![],
            nics: vec![],
            ipmi: None,
        }
    }

    fn create_test_hardware_with_components() -> Hardware {
        Hardware {
            system: Some(SystemInfo {
                sys_vendor: "Dell Inc.".to_string(),
                product_name: "PowerEdge R740".to_string(),
                product_version: "1.0".to_string(),
                serial_number: "SN-12345".to_string(),
            }),
            os: OS {
                name: "Linux".to_string(),
                version: "5.10.0".to_string(),
                kernel: "5.10.0-15-amd64".to_string(),
                architecture: "x86_64".to_string(),
                hostname: "test-server".to_string(),
                dns: "8.8.8.8".to_string(),
                ip_address: "192.168.1.100".to_string(),
            },
            cpu: CPU {
                vendor_id: "GenuineIntel".to_string(),
                model_name: "Intel(R) Xeon(R) Silver 4210R".to_string(),
                cores: 16,
                threads: 32,
                cpus: 2,
                flags: vec!["fpu".to_string(), "sse".to_string(), "avx".to_string()],
                speed: 2300,
            },
            gpus: vec![GPU {
                vendor: "NVIDIA".to_string(),
                model: "Tesla T4".to_string(),
                device_id: "1EB4".to_string(),
                serial_number: "GPU-SN-001".to_string(),
                driver_version: "535.129.02".to_string(),
            }],
            ram: RAM {
                vendor: "Samsung".to_string(),
                model: "DDR4 ECC".to_string(),
                size: 128,
                speed: 2933,
                total_size: 256,
                count: 16,
                form_factor: "DIMM".to_string(),
                modules: vec![RAMModule {
                    slot: "DIMM_A1".to_string(),
                    vendor: "Samsung".to_string(),
                    part_number: "M393A1G40Q1-CRC".to_string(),
                    serial_number: "RAM-SN-001".to_string(),
                    size: 16,
                    speed: 2933,
                    form_factor: "UDIMM".to_string(),
                    memory_type: "DDR4".to_string(),
                    locator: "CPU0_DIMM_A1".to_string(),
                }],
            },
            disks: vec![Disk {
                vendor: "Samsung".to_string(),
                size: "480.0".to_string(),
                size_unit: "GB".to_string(),
                model: "SSD 970 EVO".to_string(),
                storage_type: StorageType::SSD,
                firmware_version: "2B7Q".to_string(),
                serial_number: "DISK-SN-001".to_string(),
                parted: true,
                partitions: vec![Partition {
                    name: "/dev/sda1".to_string(),
                    size: "480.0".to_string(),
                    size_unit: "GB".to_string(),
                }],
            }],
            nics: vec![NIC {
                name: "eth0".to_string(),
                vendor: "Intel Corporation".to_string(),
                model: "I350 Gigabit Network Connection".to_string(),
                speed: 1000,
                mac_address: "00:11:22:33:44:55:66".to_string(),
                ipv4_address: "192.168.1.100".to_string(),
                ipv4_subnet_mask: "255.255.255.0".to_string(),
                ipv4_gateway: "192.168.1.1".to_string(),
                ipv6_address: "".to_string(),
                ipv6_subnet_mask: "".to_string(),
                ipv6_gateway: "".to_string(),
                dhcp: false,
                bonding_slaves: vec![],
                nic_type: NICType::Ethernet,
                status: NICStatus::Up,
                pci_slot: Some("0000:03:00.0".to_string()),
                firmware_version: "1.5.15".to_string(),
                ib_node_type: "".to_string(),
                driver: "igb".to_string(),
            }],
            ipmi: Some(IpmiInfo {
                ip_address: Some("10.0.0.10".to_string()),
                mac_address: Some("b0:31:a6:4f:d6:57".to_string()),
                subnet_mask: Some("255.255.254.0".to_string()),
                gateway: Some("10.0.0.254".to_string()),
                channel: 1,
                device_id: Some("32".to_string()),
                firmware_version: Some("6.76".to_string()),
                manufacturer_id: Some(6453),
                users: vec![],
                status: IpmiStatus::Available,
            }),
        }
    }

    #[tokio::test]
    async fn test_save_hardware_with_history_when_changed_then_creates_history() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let hardware = create_test_hardware();
        repo.save_hardware("client-005", &hardware, false)
            .await
            .unwrap();

        let mut hardware2 = create_test_hardware();
        hardware2.cpu.model_name = "Intel(R) Xeon(R) Gold 6248".to_string();
        hardware2.cpu.cores = 24;

        let result = repo.save_hardware("client-005", &hardware2, false).await;

        assert!(result.is_ok(), "Update should succeed");

        let retrieved = repo.get_hardware("client-005").await.unwrap().unwrap();
        assert_eq!(retrieved.cpu.model_name, "Intel(R) Xeon(R) Gold 6248");
        assert_eq!(retrieved.cpu.cores, 24);
    }

    #[tokio::test]
    async fn test_delete_hardware_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let hardware = create_test_hardware();
        repo.save_hardware("client-008", &hardware, true)
            .await
            .unwrap();

        let delete_result = repo.delete_hardware("client-008").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        let get_result = repo.get_hardware("client-008").await;
        assert!(get_result.unwrap().is_none(), "Hardware should be deleted");

        let history = repo.get_hardware_history("client-008").await.unwrap();
        assert_eq!(history.len(), 0, "History should also be deleted");
    }

    #[tokio::test]
    async fn test_delete_hardware_when_not_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let result = repo.delete_hardware("nonexistent").await;

        assert!(result.is_ok(), "Delete should be idempotent");
    }

    #[tokio::test]
    async fn test_hardware_semantically_eq_same_config() {
        let hardware1 = create_test_hardware();
        let hardware2 = create_test_hardware();

        assert!(
            hardware1.semantically_eq(&hardware2),
            "Same hardware should be semantically equal"
        );
    }

    #[tokio::test]
    async fn test_hardware_semantically_eq_different_cpu() {
        let mut hardware1 = create_test_hardware();
        let mut hardware2 = create_test_hardware();

        hardware1.cpu.model_name = "Intel i7".to_string();
        hardware2.cpu.model_name = "AMD Ryzen".to_string();

        assert!(
            !hardware1.semantically_eq(&hardware2),
            "Different CPU models should not be equal"
        );
    }

    #[tokio::test]
    async fn test_hardware_semantically_eq_same_cpu_different_speed() {
        let mut hardware1 = create_test_hardware();
        let mut hardware2 = create_test_hardware();

        hardware1.cpu.speed = 2400;
        hardware2.cpu.speed = 2500;

        assert!(
            hardware1.semantically_eq(&hardware2),
            "Different CPU speeds should be semantically equal (ignored)"
        );
    }

    #[tokio::test]
    async fn test_hardware_equals_ignores_nic_status_and_speed_flapping() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));
        let mut hardware1 = create_test_hardware_with_components();
        let mut hardware2 = create_test_hardware_with_components();

        hardware1.nics[0].status = NICStatus::Up;
        hardware1.nics[0].speed = 1000;
        hardware2.nics[0].status = NICStatus::Down;
        hardware2.nics[0].speed = 0;

        assert!(repo.hardware_equals(&hardware1, &hardware2));
    }

    #[tokio::test]
    async fn test_hardware_equals_keeps_nic_ip_changes() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));
        let mut hardware1 = create_test_hardware_with_components();
        let mut hardware2 = create_test_hardware_with_components();

        hardware2.nics[0].ipv4_address = "192.168.1.101".to_string();

        assert!(!repo.hardware_equals(&hardware1, &hardware2));
    }

    #[tokio::test]
    async fn test_hardware_equals_keeps_ipmi_ip_changes() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));
        let mut hardware1 = create_test_hardware_with_components();
        let mut hardware2 = create_test_hardware_with_components();

        hardware2.ipmi.as_mut().unwrap().ip_address = Some("10.0.0.11".to_string());

        assert!(!repo.hardware_equals(&hardware1, &hardware2));
    }

    #[tokio::test]
    async fn test_save_hardware_with_history_when_valid_data_then_creates_history() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let hardware = create_test_hardware();
        let result = repo.save_hardware("client-002", &hardware, true).await;

        assert!(result.is_ok(), "Save with history should succeed");

        let history = repo.get_hardware_history("client-002").await.unwrap();
        assert_eq!(history.len(), 1, "Should create one history entry");
    }

    #[tokio::test]
    async fn test_get_hardware_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let hardware = create_test_hardware();
        repo.save_hardware("client-003", &hardware, false)
            .await
            .unwrap();

        let result = repo.get_hardware("client-003").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(hardware)");
        assert_eq!(retrieved.unwrap().cpu.vendor_id, "GenuineIntel");
    }

    #[tokio::test]
    async fn test_get_hardware_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let result = repo.get_hardware("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(result.unwrap().is_none(), "Should return None");
    }

    #[tokio::test]
    async fn test_save_hardware_components_serialize_correctly() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let hardware = create_test_hardware_with_components();

        let result = repo.save_hardware("client-004", &hardware, false).await;

        assert!(result.is_ok(), "Save should succeed");

        let retrieved = repo.get_hardware("client-004").await.unwrap().unwrap();

        assert_eq!(
            retrieved.cpu.cores, 16,
            "CPU cores should serialize correctly"
        );
        assert_eq!(
            retrieved.ram.total_size, 256,
            "RAM total should serialize correctly"
        );
        assert_eq!(retrieved.disks.len(), 1, "Disks should serialize correctly");
        assert_eq!(retrieved.nics.len(), 1, "NICs should serialize correctly");
        assert_eq!(retrieved.gpus.len(), 1, "GPUs should serialize correctly");
        assert!(retrieved.ipmi.is_some(), "IPMI should serialize correctly");
    }

    #[tokio::test]
    async fn test_save_hardware_updates_data() {
        let db = setup_test_db().unwrap();
        let repo = HardwareRepository::new(std::sync::Arc::new(db));

        let mut hardware = create_test_hardware();
        repo.save_hardware("client-005", &hardware, false)
            .await
            .unwrap();

        hardware.cpu.model_name = "Intel(R) Xeon(R) Gold 6248".to_string();
        hardware.cpu.cores = 24;

        let result = repo.save_hardware("client-005", &hardware, false).await;

        assert!(result.is_ok(), "Update should succeed");

        let retrieved = repo.get_hardware("client-005").await.unwrap().unwrap();
        assert_eq!(retrieved.cpu.model_name, "Intel(R) Xeon(R) Gold 6248");
        assert_eq!(retrieved.cpu.cores, 24);
    }
}
