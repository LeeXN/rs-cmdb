use crate::config::get_config;
use crate::repository::component_repository::ComponentRepository;
use chrono::Utc;
use common::entity::hardware::Hardware;
use common::error::CmdbResult;
use common::models::{Component, ComponentStatus, ComponentType};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
use crate::tests::fixtures::*;

pub struct ComponentService {
    repo: Arc<ComponentRepository>,
}

impl ComponentService {
    pub fn new(repo: Arc<ComponentRepository>) -> Self {
        Self { repo }
    }

    fn get_valid_serial(serial: &str, client_id: &str, prefix: &str, index: usize) -> String {
        if !serial.is_empty() && serial != "N/A" && serial != "Unknown" {
            serial.to_string()
        } else {
            format!("VIRTUAL-{}-{}-{}", client_id, prefix, index)
        }
    }

    /// Process hardware info to extract and update components
    pub async fn process_hardware_info(
        &self,
        client_id: &str,
        hardware: &Hardware,
    ) -> CmdbResult<()> {
        let mut seen_serials = HashSet::new();
        let now = Utc::now().to_rfc3339();

        // 1. GPUs
        for (i, gpu) in hardware.gpus.iter().enumerate() {
            let serial = Self::get_valid_serial(&gpu.serial_number, client_id, "GPU", i);
            self.upsert_component(
                &serial,
                &gpu.model,
                Some(&gpu.vendor),
                ComponentType::GPU,
                client_id,
                &now,
            )
            .await?;
            seen_serials.insert(serial);
        }

        // 2. Disks
        for (i, disk) in hardware.disks.iter().enumerate() {
            let serial = Self::get_valid_serial(&disk.serial_number, client_id, "DISK", i);
            self.upsert_component(
                &serial,
                &disk.model,
                Some(&disk.vendor),
                ComponentType::Disk,
                client_id,
                &now,
            )
            .await?;
            seen_serials.insert(serial);
        }

        // 3. RAM Modules
        for (i, module) in hardware.ram.modules.iter().enumerate() {
            let serial = Self::get_valid_serial(&module.serial_number, client_id, "RAM", i);
            self.upsert_component(
                &serial,
                &format!("{} {}GB", module.memory_type, module.size),
                Some(&module.vendor),
                ComponentType::Memory,
                client_id,
                &now,
            )
            .await?;
            seen_serials.insert(serial);
        }

        // 4. NICs (Use MAC as Serial)
        for nic in &hardware.nics {
            if !nic.mac_address.is_empty() {
                self.upsert_component(
                    &nic.mac_address,
                    &nic.model,
                    Some(&nic.vendor),
                    ComponentType::NetworkCard,
                    client_id,
                    &now,
                )
                .await?;
                seen_serials.insert(nic.mac_address.clone());
            }
        }

        // 5. Handle removed components
        let existing_components = self.repo.find_by_client_id(client_id).await?;
        let grace_period = get_config().component_missing_grace_period_hours;

        for comp in existing_components {
            if !seen_serials.contains(&comp.serial_number) {
                let mut updated = comp.clone();

                if let Some(missing_since_str) = &updated.missing_since {
                    // Already marked as missing, check if grace period expired
                    if let Ok(missing_since) =
                        chrono::DateTime::parse_from_rfc3339(missing_since_str)
                    {
                        let duration = Utc::now().signed_duration_since(missing_since);
                        if duration.num_hours() >= grace_period as i64 {
                            // Expired, detach
                            updated.client_id = None;
                            updated.status = ComponentStatus::Unknown;
                            updated.location = Some("Unknown".to_string());
                            updated.missing_since = None;
                            updated.updated_at = now.clone();
                            self.repo.save(&updated).await?;
                        }
                    }
                } else {
                    // First time missing, mark it
                    updated.missing_since = Some(now.clone());
                    updated.updated_at = now.clone();
                    self.repo.save(&updated).await?;
                }
            }
        }

        Ok(())
    }

    async fn upsert_component(
        &self,
        serial: &str,
        model: &str,
        vendor: Option<&str>,
        ctype: ComponentType,
        client_id: &str,
        now: &str,
    ) -> CmdbResult<()> {
        if let Some(mut comp) = self.repo.find_by_serial(serial).await? {
            // Update existing
            // Only update if something changed to avoid unnecessary writes
            if comp.client_id.as_deref() != Some(client_id)
                || comp.status != ComponentStatus::InUse
                || comp.missing_since.is_some()
            {
                comp.client_id = Some(client_id.to_string());
                comp.status = ComponentStatus::InUse;
                comp.updated_at = now.to_string();
                comp.missing_since = None; // Clear missing flag
                // We could update model/vendor here too if we trust the new data more
                comp.model = model.to_string();
                comp.vendor = vendor.map(|s| s.to_string());
                self.repo.save(&comp).await?;
            }
        } else {
            // Create new
            let new_comp = Component {
                id: Uuid::new_v4().to_string(),
                serial_number: serial.to_string(),
                model: model.to_string(),
                vendor: vendor.map(|s| s.to_string()),
                component_type: ctype,
                status: ComponentStatus::InUse,
                client_id: Some(client_id.to_string()),
                client_hostname: None,
                location: None,
                purchase_date: None,
                warranty_expiration: None,
                missing_since: None,
                created_at: now.to_string(),
                updated_at: now.to_string(),
            };
            self.repo.save(&new_comp).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::component_repository::ComponentRepository;
    use std::sync::Arc;

    use common::models::{Component, ComponentStatus, ComponentType};
    use uuid::Uuid;

    fn create_test_component(serial: &str, client_id: &str) -> Component {
        Component {
            id: Uuid::new_v4().to_string(),
            serial_number: serial.to_string(),
            model: "Test Model".to_string(),
            vendor: Some("Test Vendor".to_string()),
            component_type: ComponentType::GPU,
            status: ComponentStatus::InUse,
            client_id: Some(client_id.to_string()),
            client_hostname: None,
            location: None,
            purchase_date: None,
            warranty_expiration: None,
            missing_since: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_get_valid_serial_with_valid_serial() {
        let serial = "SN-12345";
        let result = ComponentService::get_valid_serial(serial, "client-1", "GPU", 0);
        assert_eq!(result, serial.to_string());
    }

    #[tokio::test]
    async fn test_get_valid_serial_with_empty_serial() {
        let serial = "";
        let result = ComponentService::get_valid_serial(serial, "client-1", "GPU", 0);
        assert_eq!(result, "VIRTUAL-client-1-GPU-0");
    }

    #[tokio::test]
    async fn test_get_valid_serial_with_na_serial() {
        let serial = "N/A";
        let result = ComponentService::get_valid_serial(serial, "client-1", "GPU", 1);
        assert_eq!(result, "VIRTUAL-client-1-GPU-1");
    }

    #[tokio::test]
    async fn test_get_valid_serial_with_unknown_serial() {
        let serial = "Unknown";
        let result = ComponentService::get_valid_serial(serial, "client-1", "DISK", 2);
        assert_eq!(result, "VIRTUAL-client-1-DISK-2");
    }

    #[tokio::test]
    async fn test_process_hardware_info_with_new_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo);

        let hardware = create_test_hardware_info("client-1");
        let result = service.process_hardware_info("client-1", &hardware).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_hardware_info_creates_gpu_components() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let hardware = create_test_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        let gpu_components: Vec<_> = components
            .iter()
            .filter(|c| c.component_type == ComponentType::GPU)
            .collect();

        assert!(!gpu_components.is_empty());
        assert_eq!(gpu_components[0].client_id.as_deref(), Some("client-1"));
        assert_eq!(gpu_components[0].status, ComponentStatus::InUse);
    }

    #[tokio::test]
    async fn test_process_hardware_info_creates_disk_components() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let hardware = create_test_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        let disk_components: Vec<_> = components
            .iter()
            .filter(|c| c.component_type == ComponentType::Disk)
            .collect();

        assert!(!disk_components.is_empty());
        assert_eq!(disk_components[0].client_id.as_deref(), Some("client-1"));
    }

    #[tokio::test]
    async fn test_process_hardware_info_creates_memory_components() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let hardware = create_test_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        let memory_components: Vec<_> = components
            .iter()
            .filter(|c| c.component_type == ComponentType::Memory)
            .collect();

        assert!(!memory_components.is_empty());
        assert_eq!(memory_components[0].client_id.as_deref(), Some("client-1"));
    }

    #[tokio::test]
    async fn test_process_hardware_info_creates_network_components() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let hardware = create_test_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        let nic_components: Vec<_> = components
            .iter()
            .filter(|c| c.component_type == ComponentType::NetworkCard)
            .collect();

        assert!(!nic_components.is_empty());
    }

    #[tokio::test]
    async fn test_process_hardware_info_updates_existing_component() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let hardware = create_test_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        let component_count = components.len();

        service
            .process_hardware_info("client-1", &hardware)
            .await
            .unwrap();

        let components_after = repo.find_by_client_id("client-1").await.unwrap();
        assert_eq!(components_after.len(), component_count);
    }

    #[tokio::test]
    async fn test_process_hardware_info_marks_missing_components() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let hardware1 = create_test_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware1)
            .await
            .unwrap();

        let hardware2 = create_minimal_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware2)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        let missing_components: Vec<_> = components
            .iter()
            .filter(|c| c.missing_since.is_some())
            .collect();

        assert!(!missing_components.is_empty());
    }

    #[tokio::test]
    async fn test_process_hardware_info_detaches_expired_components() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let hardware1 = create_test_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware1)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        if let Some(comp) = components.first() {
            let mut updated_comp = comp.clone();
            let old_time = chrono::Utc::now() - chrono::Duration::hours(25);
            updated_comp.missing_since = Some(old_time.to_rfc3339());
            repo.save(&updated_comp).await.unwrap();
        }

        let hardware2 = create_minimal_hardware_info("client-1");
        service
            .process_hardware_info("client-1", &hardware2)
            .await
            .unwrap();

        let all_components = repo.list_all().await.unwrap();
        let detached: Vec<_> = all_components
            .iter()
            .filter(|c| c.client_id.is_none())
            .collect();

        assert!(!detached.is_empty());
    }

    #[tokio::test]
    async fn test_process_hardware_info_with_empty_hardware() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo);

        let hardware = create_empty_hardware_info();
        let result = service.process_hardware_info("client-1", &hardware).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_hardware_info_with_multiple_nics() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let repo = Arc::new(ComponentRepository::new(Arc::clone(&db_arc)));
        let service = ComponentService::new(repo.clone());

        let mut hardware = create_test_hardware_info("client-1");
        hardware.nics = vec![hardware.nics[0].clone(), {
            let mut nic = hardware.nics[0].clone();
            nic.mac_address = "00:11:22:33:44:66".to_string();
            nic
        }];

        service
            .process_hardware_info("client-1", &hardware)
            .await
            .unwrap();

        let components = repo.find_by_client_id("client-1").await.unwrap();
        let nic_components: Vec<_> = components
            .iter()
            .filter(|c| c.component_type == ComponentType::NetworkCard)
            .collect();

        assert_eq!(nic_components.len(), 2);
    }
}
