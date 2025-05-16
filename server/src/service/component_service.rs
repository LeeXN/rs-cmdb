use std::sync::Arc;
use std::collections::HashSet;
use common::error::CmdbResult;
use common::models::{Component, ComponentType, ComponentStatus};
use common::entity::hardware::Hardware;
use crate::repository::component_repository::ComponentRepository;
use crate::config::get_config;
use uuid::Uuid;
use chrono::Utc;

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
    pub async fn process_hardware_info(&self, client_id: &str, hardware: &Hardware) -> CmdbResult<()> {
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
                &now
            ).await?;
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
                &now
            ).await?;
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
                &now
            ).await?;
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
                    &now
                ).await?;
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
                    if let Ok(missing_since) = chrono::DateTime::parse_from_rfc3339(missing_since_str) {
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
        now: &str
    ) -> CmdbResult<()> {
        if let Some(mut comp) = self.repo.find_by_serial(serial).await? {
            // Update existing
            // Only update if something changed to avoid unnecessary writes
            if comp.client_id.as_deref() != Some(client_id) || comp.status != ComponentStatus::InUse || comp.missing_since.is_some() {
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
