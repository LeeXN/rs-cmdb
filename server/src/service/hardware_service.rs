use crate::cache::CachedClientRepository;
use crate::queue::MessageQueue;
use crate::repository::hardware_repository::HardwareRepository;
use crate::service::component_service::ComponentService;
use common::entity::hardware::NICType;
use common::error::{CmdbError, CmdbResult};
use common::models::{ClientHardwareInfo, PullResponse};
use ipnet::IpNet;
use std::str::FromStr;
use std::sync::Arc;

#[cfg(test)]
use crate::tests::fixtures::*;

/// Service for hardware operations
pub struct HardwareService {
    client_repo: Arc<CachedClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
    component_service: Arc<ComponentService>,
    #[allow(dead_code)]
    message_queue: Arc<dyn MessageQueue>,
    primary_ip_subnet: Option<IpNet>,
}

impl HardwareService {
    /// Create a new hardware service
    pub fn new(
        client_repo: Arc<CachedClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
        component_service: Arc<ComponentService>,
        message_queue: Arc<dyn MessageQueue>,
        primary_ip_subnet: Option<IpNet>,
    ) -> Self {
        Self {
            client_repo,
            hardware_repo,
            component_service,
            message_queue,
            primary_ip_subnet,
        }
    }

    /// Auto-detect primary IP from NICs matching the configured subnet
    pub fn detect_primary_ip(&self, nics: &[common::entity::hardware::NIC]) -> Option<String> {
        let subnet = self.primary_ip_subnet.as_ref()?;
        // First pass: look for Ethernet NICs matching the subnet
        for nic in nics {
            if nic.nic_type == NICType::Ethernet {
                if let Ok(ip) = std::net::IpAddr::from_str(&nic.ipv4_address) {
                    if subnet.contains(&ip) {
                        return Some(nic.ipv4_address.clone());
                    }
                }
            }
        }
        // Second pass: any NIC matching the subnet
        for nic in nics {
            if let Ok(ip) = std::net::IpAddr::from_str(&nic.ipv4_address) {
                if subnet.contains(&ip) {
                    return Some(nic.ipv4_address.clone());
                }
            }
        }
        None
    }

    /// Process hardware info from a client
    pub async fn process_hardware_info(&self, hardware_info: ClientHardwareInfo) -> CmdbResult<()> {
        let client_id = &hardware_info.client_id;

        // Check if client exists
        if let Ok(false) = self.client_repo.exists(client_id).await {
            return Err(CmdbError::NotFound(format!(
                "Client {} not found",
                client_id
            )));
        }

        // Update last seen timestamp
        self.client_repo.update_last_seen(client_id).await?;

        // Save hardware info if available
        if let Some(hardware) = hardware_info.hardware {
            self.hardware_repo
                .save_hardware_with_timestamp(
                    client_id,
                    &hardware,
                    true,
                    Some(&hardware_info.collected_at),
                )
                .await?;

            // Extract and update components
            if let Err(e) = self
                .component_service
                .process_hardware_info(client_id, &hardware)
                .await
            {
                eprintln!(
                    "Error processing components for client {}: {}",
                    client_id, e
                );
            }

            // Auto-detect primary IP from NICs
            if self.primary_ip_subnet.is_some() {
                if let Ok(Some(client)) = self.client_repo.get(client_id).await {
                    if client.primary_ip.is_none() {
                        if let Some(primary_ip) = self.detect_primary_ip(&hardware.nics) {
                            if let Err(e) = self
                                .client_repo
                                .update_primary_ip(client_id, &primary_ip)
                                .await
                            {
                                eprintln!(
                                    "Error updating primary IP for client {}: {}",
                                    client_id, e
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a pull response from a client
    pub async fn process_pull_response(&self, response: PullResponse) -> CmdbResult<()> {
        // Extract request ID and client ID from response
        let request_id = &response.request_id;

        // In a real-world scenario, we would look up the original pull request
        // to get the client ID. For simplicity, we assume it's embedded in the request ID.
        let parts: Vec<&str> = request_id.split(':').collect();
        if parts.len() < 2 {
            return Err(CmdbError::Validation(format!(
                "Invalid request ID format: {}",
                request_id
            )));
        }

        let client_id = parts[0];

        // Check if client exists
        if let Ok(false) = self.client_repo.exists(client_id).await {
            return Err(CmdbError::NotFound(format!(
                "Client {} not found",
                client_id
            )));
        }

        // Update last seen timestamp
        self.client_repo.update_last_seen(client_id).await?;

        // Save hardware info if available and status is success
        if response.status == "success"
            && let Some(hardware) = response.hardware
        {
            self.hardware_repo
                .save_hardware(client_id, &hardware, true)
                .await?;

            // Extract and update components
            if let Err(e) = self
                .component_service
                .process_hardware_info(client_id, &hardware)
                .await
            {
                eprintln!(
                    "Error processing components for client {}: {}",
                    client_id, e
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{CacheConfigs, CachedClientRepository};
    use crate::repository::{
        client_repository::ClientRepository, component_repository::ComponentRepository,
        hardware_repository::HardwareRepository,
    };
    use common::entity::hardware::NIC;
    use std::sync::Arc;

    fn create_service(db: Arc<dyn crate::db::Database>) -> HardwareService {
        let client_repo_inner = Arc::new(ClientRepository::new(Arc::clone(&db)));
        let cache_configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(
            client_repo_inner.clone(),
            &cache_configs,
        ));

        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        HardwareService::new(
            client_repo,
            hardware_repo,
            component_service,
            mock_queue,
            None,
        )
    }

    #[tokio::test]
    async fn test_hardware_service_creation() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let _service = create_service(db_arc);
    }

    #[tokio::test]
    async fn test_process_hardware_info_with_valid_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client_repo = ClientRepository::new(db_arc.clone());
        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let hardware_info = create_client_hardware_info("client-1");
        let result = service.process_hardware_info(hardware_info).await;

        assert!(result.is_ok());
    }

    fn create_service_with_subnet(
        db: Arc<dyn crate::db::Database>,
        subnet: &str,
    ) -> HardwareService {
        let client_repo_inner = Arc::new(ClientRepository::new(Arc::clone(&db)));
        let cache_configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(
            client_repo_inner.clone(),
            &cache_configs,
        ));

        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());
        let parsed_subnet = IpNet::from_str(subnet).expect("Invalid subnet CIDR");

        HardwareService::new(
            client_repo,
            hardware_repo,
            component_service,
            mock_queue,
            Some(parsed_subnet),
        )
    }

    fn make_nic(name: &str, ip: &str, nic_type: NICType) -> NIC {
        NIC {
            name: name.to_string(),
            vendor: "Intel".to_string(),
            model: "I350".to_string(),
            speed: 1000,
            mac_address: "00:11:22:33:44:55".to_string(),
            ipv4_address: ip.to_string(),
            ipv4_subnet_mask: "255.255.255.0".to_string(),
            ipv4_gateway: "192.168.1.1".to_string(),
            ipv6_address: String::new(),
            ipv6_subnet_mask: String::new(),
            ipv6_gateway: String::new(),
            dhcp: true,
            bonding_slaves: vec![],
            nic_type,
            status: common::entity::hardware::NICStatus::Up,
            pci_slot: Some("0000:99:00.0".to_string()),
            firmware_version: String::new(),
            ib_node_type: String::new(),
            driver: "igb".to_string(),
        }
    }

    #[test]
    fn test_detect_primary_ip_no_subnet() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);
        assert_eq!(service.detect_primary_ip(&[]), None);
    }

    #[test]
    fn test_detect_primary_ip_ethernet_match() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service_with_subnet(db_arc, "192.168.0.0/16");
        let nics = vec![make_nic("eth0", "192.168.1.100", NICType::Ethernet)];
        assert_eq!(
            service.detect_primary_ip(&nics),
            Some("192.168.1.100".to_string())
        );
    }

    #[test]
    fn test_detect_primary_ip_prefers_ethernet() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service_with_subnet(db_arc, "10.0.0.0/8");
        let nics = vec![
            make_nic("bond0", "10.0.0.200", NICType::Bonding),
            make_nic("eth0", "10.0.0.100", NICType::Ethernet),
        ];
        assert_eq!(
            service.detect_primary_ip(&nics),
            Some("10.0.0.100".to_string())
        );
    }

    #[test]
    fn test_detect_primary_ip_no_match() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service_with_subnet(db_arc, "10.0.0.0/8");
        let nics = vec![make_nic("eth0", "192.168.1.100", NICType::Ethernet)];
        assert_eq!(service.detect_primary_ip(&nics), None);
    }

    #[test]
    fn test_detect_primary_ip_empty_nics() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service_with_subnet(db_arc, "10.0.0.0/8");
        assert_eq!(service.detect_primary_ip(&[]), None);
    }

    #[tokio::test]
    async fn test_process_hardware_info_with_nonexistent_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let hardware_info = create_client_hardware_info("nonexistent");
        let result = service.process_hardware_info(hardware_info).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_hardware_info_updates_last_seen() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client_repo = ClientRepository::new(db_arc.clone());
        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let initial_client = client_repo.get("client-1").await.unwrap().unwrap();
        let initial_last_seen = initial_client.last_seen.clone();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let hardware_info = create_client_hardware_info("client-1");
        service.process_hardware_info(hardware_info).await.unwrap();

        let updated_client = client_repo.get("client-1").await.unwrap().unwrap();
        let updated_last_seen = updated_client.last_seen;

        assert_ne!(initial_last_seen, updated_last_seen);
    }

    #[tokio::test]
    async fn test_process_hardware_info_saves_hardware() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client_repo = ClientRepository::new(db_arc.clone());
        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let hardware_info = create_client_hardware_info("client-1");
        service.process_hardware_info(hardware_info).await.unwrap();

        let hardware_repo = HardwareRepository::new(db_arc);
        let hardware = hardware_repo.get_hardware("client-1").await.unwrap();
        assert!(hardware.is_some());
    }

    #[tokio::test]
    async fn test_process_hardware_info_without_hardware() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client_repo = ClientRepository::new(db_arc.clone());
        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let mut hardware_info = create_client_hardware_info("client-1");
        hardware_info.hardware = None;

        let result = service.process_hardware_info(hardware_info).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_pull_response_with_success() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client_repo = ClientRepository::new(db_arc.clone());
        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let pull_response = create_pull_response("client-1", "success");
        let result = service.process_pull_response(pull_response).await;

        assert!(result.is_ok());

        let hardware_repo = HardwareRepository::new(db_arc);
        let hardware = hardware_repo.get_hardware("client-1").await.unwrap();
        assert!(hardware.is_some());
    }

    #[tokio::test]
    async fn test_process_pull_response_with_error() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client_repo = ClientRepository::new(db_arc.clone());
        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let pull_response = create_pull_response("client-1", "error");
        let result = service.process_pull_response(pull_response).await;

        assert!(result.is_ok());

        let hardware_repo = HardwareRepository::new(db_arc);
        let hardware = hardware_repo.get_hardware("client-1").await.unwrap();
        assert!(hardware.is_none());
    }

    #[tokio::test]
    async fn test_process_pull_response_with_invalid_request_id() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let pull_response = PullResponse {
            request_id: "invalid-request-id".to_string(),
            status: "success".to_string(),
            hardware: None,
            error: None,
        };

        let result = service.process_pull_response(pull_response).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_pull_response_with_nonexistent_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let pull_response = create_pull_response("nonexistent", "success");
        let result = service.process_pull_response(pull_response).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_hardware_info_auto_detects_primary_ip() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service_with_subnet(db_arc.clone(), "192.168.0.0/16");

        let client_repo = ClientRepository::new(db_arc.clone());
        let mut client = create_test_client("client-1");
        client.primary_ip = None;
        client_repo.save(&client).await.unwrap();

        let hardware_info = create_client_hardware_info("client-1");
        service.process_hardware_info(hardware_info).await.unwrap();

        let updated = client_repo.get("client-1").await.unwrap().unwrap();
        assert_eq!(updated.primary_ip, Some("192.168.1.1".to_string()));
    }

    #[tokio::test]
    async fn test_process_hardware_info_does_not_override_manual_primary_ip() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service_with_subnet(db_arc.clone(), "192.168.0.0/16");

        let client_repo = ClientRepository::new(db_arc.clone());
        let mut client = create_test_client("client-1");
        client.primary_ip = Some("10.0.0.50".to_string());
        client_repo.save(&client).await.unwrap();

        let hardware_info = create_client_hardware_info("client-1");
        service.process_hardware_info(hardware_info).await.unwrap();

        let updated = client_repo.get("client-1").await.unwrap().unwrap();
        assert_eq!(updated.primary_ip, Some("10.0.0.50".to_string()));
    }
}
