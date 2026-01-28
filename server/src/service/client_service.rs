//! Client service for client business logic
//!
//! This service uses DAO layer for data access, reducing coupling
//! and simplifying business logic.

use crate::dao::{ClientDao, RackDao};
use crate::repository::hardware_repository::HardwareRepository;
use crate::validation::validate_ip_address;
use common::error::{CmdbError, CmdbResult};
use common::models::Client;
use std::sync::Arc;
use tracing::{info, instrument, warn};

#[cfg(test)]
use crate::tests::fixtures::*;

/// Service for client operations
pub struct ClientService {
    client_dao: Arc<ClientDao>,
    rack_dao: Arc<RackDao>,
    hardware_repo: Arc<HardwareRepository>,
}

impl ClientService {
    /// Create a new client service using DAO layer
    pub fn new(
        client_dao: Arc<ClientDao>,
        rack_dao: Arc<RackDao>,
        hardware_repo: Arc<HardwareRepository>,
    ) -> Self {
        Self {
            client_dao,
            rack_dao,
            hardware_repo,
        }
    }

    /// Create a new client service from repositories (backward compatibility)
    pub fn from_repositories(
        client_repo: Arc<crate::repository::client_repository::ClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
        rack_repo: Arc<crate::repository::rack_repository::RackRepository>,
    ) -> Self {
        Self {
            client_dao: Arc::new(ClientDao::new(client_repo.clone(), hardware_repo.clone())),
            rack_dao: Arc::new(RackDao::new(rack_repo, client_repo)),
            hardware_repo,
        }
    }

    /// Import clients from a list
    #[instrument(skip(self, clients))]
    pub async fn import_clients(&self, clients: Vec<Client>) -> CmdbResult<usize> {
        let mut count = 0;
        for client in clients {
            // Validate Rack Assignment using DAO
            if let Some(rack_id) = &client.rack
                && !rack_id.is_empty()
            {
                // First, check if rack exists
                if self.rack_dao.get(rack_id).await?.is_none() {
                    return Err(CmdbError::Validation(format!(
                        "Rack {} not found for client {}",
                        rack_id, client.hostname
                    )));
                }

                // Validate position using DAO if unit_position is set
                if let Some(pos_str) = &client.unit_position
                    && let Ok(pos) = pos_str.parse::<u32>()
                {
                    let height = client.u_height.unwrap_or(1);
                    self.rack_dao
                        .validate_position(rack_id, pos, height, Some(&client.id))
                        .await?;
                }
            }

            // Save or Update
            if self.client_dao.get(&client.id).await?.is_some() {
                self.client_dao.save(&client).await?;
            } else {
                let mut new_client = client.clone();
                if new_client.id.is_empty() {
                    new_client.id = uuid::Uuid::new_v4().to_string();
                }
                self.client_dao.save(&new_client).await?;
            }
            count += 1;
        }
        Ok(count)
    }

    /// Register a new client
    ///
    /// If client_id is provided, it will be used. Otherwise, a new UUID will be generated.
    #[instrument(
        skip(
            self,
            hostname,
            ip_address,
            sys_vendor,
            product_name,
            serial_number,
            os
        ),
        fields(client_id)
    )]
    pub async fn register_client(
        &self,
        hostname: &str,
        ip_address: &str,
        sys_vendor: &str,
        product_name: &str,
        serial_number: &str,
        os: &str,
        client_id: Option<String>,
    ) -> CmdbResult<Client> {
        // Create a new client with given or generated ID
        let mut client = Client::new(hostname.to_string(), ip_address.to_string());

        // Use provided client ID if available
        if let Some(id) = client_id {
            client.id = id;
        } else {
            // Try to find existing client by serial number if client_id is not provided
            // Note: This uses the underlying repo since find_by_serial is repo-specific
            // In a full refactor, this would also be in the DAO
            if let Ok(Some(existing)) = self
                .client_dao
                .get_by_serial(serial_number)
                .await
            {
                info!(
                    "Found existing client by serial number: {} -> {}",
                    serial_number, existing.id
                );
                client.id = existing.id;
            }
        }

        // Check if client already exists
        if self.client_dao.get(&client.id).await?.is_some() {
            // Update the client information
            if let Some(mut existing_client) = self.client_dao.get(&client.id).await? {
                existing_client.hostname = hostname.to_string();
                existing_client.ip_address = ip_address.to_string();
                existing_client.sys_vendor = Some(sys_vendor.to_string());
                existing_client.product_name = Some(product_name.to_string());
                existing_client.serial_number = Some(serial_number.to_string());
                existing_client.os = Some(os.to_string());
                existing_client.update_last_seen();

                self.client_dao.save(&existing_client).await?;
                info!(
                    "Client updated: {} ({})",
                    existing_client.hostname, existing_client.id
                );
                return Ok(existing_client);
            }
        }

        // Save the new client
        self.client_dao.save(&client).await?;
        info!("New client registered: {} ({})", client.hostname, client.id);

        Ok(client)
    }

    /// Delete a client and associated hardware information
    /// Also attempts to stop the client service remotely
    #[instrument(skip(self))]
    pub async fn delete_client(&self, client_id: &str) -> CmdbResult<()> {
        // Get client information first
        let client = self.client_dao.get(client_id).await?.ok_or_else(|| {
            CmdbError::NotFound(format!("Client {} not found", client_id))
        })?;

        info!(
            "Deleting client: {} ({})",
            client.hostname, client.ip_address
        );

        // Attempt to stop the client service remotely
        if let Err(e) = self.stop_client_service(&client).await {
            warn!(
                "Failed to stop client service for {}: {}",
                client.hostname, e
            );
        }

        // Delete hardware data
        match self.hardware_repo.delete_hardware(client_id).await {
            Ok(_) => info!("Hardware data deleted for client {}", client_id),
            Err(e) => warn!(
                "Failed to delete hardware data for client {}: {}",
                client_id, e
            ),
        }

        // Delete client from database
        self.client_dao.delete(client_id).await?;
        info!("Client {} deleted successfully from database", client_id);

        Ok(())
    }

    /// Attempt to stop the client service remotely
    #[instrument(skip(self, client), fields(hostname = %client.hostname, ip = %client.ip_address))]
    async fn stop_client_service(&self, client: &Client) -> CmdbResult<()> {
        info!("Attempting to stop service on client");

        // For Linux systems, try to stop systemd service via SSH
        if client
            .os
            .as_ref()
            .is_some_and(|os| os.to_lowercase().contains("linux"))
        {
            return self.stop_linux_service(client).await;
        }

        // For Windows systems, try to stop service via PowerShell remoting
        if client
            .os
            .as_ref()
            .is_some_and(|os| os.to_lowercase().contains("windows"))
        {
            return self.stop_windows_service(client).await;
        }

        warn!(
            "Unsupported OS for remote service management: {:?}",
            client.os
        );
        Ok(())
    }

    /// Stop Linux service via SSH (if configured)
    async fn stop_linux_service(&self, client: &Client) -> CmdbResult<()> {
        // Validate IP address before using it in SSH command
        let validated_ip = validate_ip_address(&client.ip_address)?;
        info!("Validated IP address for SSH: {}", validated_ip);

        let commands = vec![
            "systemctl stop rs-cmdb-client",
            "systemctl disable rs-cmdb-client",
        ];

        let config = crate::config::get_config();
        let known_hosts_file = config.ssh_known_hosts_file.as_deref();

        for cmd in commands {
            info!("Executing on {}: {}", client.hostname, cmd);

            let mut ssh_cmd = tokio::process::Command::new("ssh");
            ssh_cmd
                .arg("-o")
                .arg("ConnectTimeout=30")
                .arg("-o")
                .arg("BatchMode=yes")
                .arg("-o")
                .arg("StrictHostKeyChecking=yes");

            if let Some(known_hosts) = known_hosts_file {
                ssh_cmd
                    .arg("-o")
                    .arg(format!("UserKnownHostsFile={}", known_hosts));
            }

            // Use validated IP address
            ssh_cmd.arg(&validated_ip).arg(cmd);

            let output = ssh_cmd.output().await;

            match output {
                Ok(result) => {
                    if result.status.success() {
                        info!("Command executed successfully on {}", client.hostname);
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        warn!("Command failed on {}: {}", client.hostname, stderr);

                        if stderr.contains("Host key verification failed")
                            || stderr.contains("Could not resolve hostname")
                        {
                            return Err(CmdbError::Validation(format!(
                                "SSH host key verification failed for {}. Please add the host to your known_hosts file.",
                                client.hostname
                            )));
                        }
                    }
                }
                Err(e) => {
                    let error_msg = format!("SSH connection failed to {}: {}", client.hostname, e);
                    warn!("{}", error_msg);

                    if e.kind() == std::io::ErrorKind::NotFound {
                        return Err(CmdbError::Validation("SSH command not found. Please ensure OpenSSH client is installed to manage remote services.".to_string()));
                    }
                    return Err(CmdbError::Internal(error_msg));
                }
            }
        }

        Ok(())
    }

    /// Stop Windows service via PowerShell remoting (if configured)
    async fn stop_windows_service(&self, client: &Client) -> CmdbResult<()> {
        info!("Attempting to stop Windows service on {}", client.hostname);
        warn!(
            "Windows service stopping not implemented yet for {}",
            client.hostname
        );
        Ok(())
    }

    /// Get a client by ID
    #[allow(dead_code)]
    pub async fn get_client(&self, client_id: &str) -> CmdbResult<Option<Client>> {
        self.client_dao.get(client_id).await
    }

    /// List all clients
    #[allow(dead_code)]
    pub async fn list_clients(&self) -> CmdbResult<Vec<Client>> {
        self.client_dao.list_all().await
    }

    /// Update client last seen timestamp
    pub async fn update_last_seen(&self, client_id: &str) -> CmdbResult<()> {
        // This uses the underlying repo for now - could be added to DAO
        if let Some(mut client) = self.client_dao.get(client_id).await? {
            client.update_last_seen();
            self.client_dao.save(&client).await?;
        }
        Ok(())
    }
}

// Extension methods for ClientDao to support existing functionality
impl ClientDao {
    /// Find client by serial number
    pub async fn get_by_serial(&self, serial: &str) -> CmdbResult<Option<Client>> {
        let all_clients = self.list_all().await?;
        Ok(all_clients
            .into_iter()
            .find(|c| c.serial_number.as_deref() == Some(serial)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{
        client_repository::ClientRepository, hardware_repository::HardwareRepository,
        rack_repository::RackRepository,
    };
    use std::sync::Arc;

    fn create_service(
        db: Arc<dyn crate::db::Database>,
    ) -> ClientService {
        let client_repo = Arc::new(ClientRepository::new(db.clone()));
        let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));
        let rack_repo = Arc::new(RackRepository::new(db.clone()));
        ClientService::from_repositories(client_repo, hardware_repo, rack_repo)
    }

    #[tokio::test]
    async fn test_client_service_creation() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let _service = create_service(db_arc);
    }

    #[tokio::test]
    async fn test_import_clients_with_valid_data() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let rack = create_test_rack("rack-1");
        let rack_repo = RackRepository::new(db_arc);
        rack_repo.save(&rack).await.unwrap();

        let client = create_test_client("client-1");
        let result = service.import_clients(vec![client]).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_import_clients_with_nonexistent_rack() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let mut client = create_test_client("client-1");
        client.rack = Some("nonexistent-rack".to_string());

        let result = service.import_clients(vec![client]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_clients_with_valid_unit_position() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let mut rack = create_test_rack("rack-1");
        rack.height_u = 10;
        let rack_repo = RackRepository::new(db_arc.clone());
        rack_repo.save(&rack).await.unwrap();

        let mut client = create_test_client("client-1");
        client.rack = Some("rack-1".to_string());
        client.unit_position = Some("5".to_string());
        client.u_height = Some(2);

        let result = service.import_clients(vec![client]).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_clients_with_invalid_unit_position() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let rack = create_test_rack("rack-1");
        let rack_repo = RackRepository::new(db_arc.clone());
        rack_repo.save(&rack).await.unwrap();

        let mut client = create_test_client("client-1");
        client.rack = Some("rack-1".to_string());
        client.unit_position = Some("40".to_string());
        client.u_height = Some(5);

        let result = service.import_clients(vec![client]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_clients_with_overlapping_positions() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let rack = create_test_rack("rack-1");
        let rack_repo = RackRepository::new(db_arc.clone());
        rack_repo.save(&rack).await.unwrap();

        let mut client1 = create_test_client("client-1");
        client1.rack = Some("rack-1".to_string());
        client1.unit_position = Some("1".to_string());
        client1.u_height = Some(2);

        let mut client2 = create_test_client("client-2");
        client2.rack = Some("rack-1".to_string());
        client2.unit_position = Some("2".to_string());
        client2.u_height = Some(2);

        let client_repo = ClientRepository::new(db_arc);
        client_repo.save(&client1).await.unwrap();

        let result = service.import_clients(vec![client2]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_new_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let client = service
            .register_client(
                "test-host",
                "192.168.1.1",
                "Dell",
                "PowerEdge",
                "SN12345",
                "Linux",
                None,
            )
            .await;

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.hostname, "test-host");
        assert_eq!(client.ip_address, "192.168.1.1");
    }

    #[tokio::test]
    async fn test_register_client_with_provided_id() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let custom_id = "custom-client-id-123".to_string();
        let client = service
            .register_client(
                "test-host",
                "192.168.1.1",
                "Dell",
                "PowerEdge",
                "SN12345",
                "Linux",
                Some(custom_id.clone()),
            )
            .await;

        assert!(client.is_ok());
        assert_eq!(client.unwrap().id, custom_id);
    }

    #[tokio::test]
    async fn test_register_existing_client_by_serial() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let mut client = create_test_client("test-client");
        client.serial_number = Some("SN12345".to_string());
        let client_repo = ClientRepository::new(db_arc.clone());
        client_repo.save(&client).await.unwrap();

        let result = service
            .register_client(
                "new-hostname",
                "192.168.1.2",
                "Dell",
                "PowerEdge",
                "SN12345",
                "Linux",
                None,
            )
            .await;

        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.id, "test-client");
        assert_eq!(client.hostname, "new-hostname");
    }

    #[tokio::test]
    async fn test_get_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client = create_test_client("test-client");
        let client_repo = ClientRepository::new(db_arc);
        client_repo.save(&client).await.unwrap();

        let result = service.get_client("test-client").await;

        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-client");
    }

    #[tokio::test]
    async fn test_list_clients() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client1 = create_test_client("client-1");
        let client2 = create_test_client("client-2");
        let client_repo = ClientRepository::new(db_arc);
        client_repo.save(&client1).await.unwrap();
        client_repo.save(&client2).await.unwrap();

        let result = service.list_clients().await;

        assert!(result.is_ok());
        let clients = result.unwrap();
        assert_eq!(clients.len(), 2);
    }

    #[tokio::test]
    async fn test_update_last_seen() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client = create_test_client("test-client");
        let client_repo = ClientRepository::new(db_arc.clone());
        client_repo.save(&client).await.unwrap();

        let initial_client = client_repo.get("test-client").await.unwrap().unwrap();
        let initial_last_seen = initial_client.last_seen.clone();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        service.update_last_seen("test-client").await.unwrap();

        let updated_client = client_repo.get("test-client").await.unwrap().unwrap();
        let updated_last_seen = updated_client.last_seen;

        assert_ne!(initial_last_seen, updated_last_seen);
    }

    #[tokio::test]
    async fn test_delete_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client = create_test_client("test-client");
        let client_repo = ClientRepository::new(db_arc.clone());
        client_repo.save(&client).await.unwrap();

        let result = service.delete_client("test-client").await;

        assert!(result.is_ok());
        assert!(client_repo.get("test-client").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let result = service.delete_client("nonexistent-client").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_client_removes_hardware() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let client = create_test_client("test-client");
        let client_repo = ClientRepository::new(db_arc.clone());
        client_repo.save(&client).await.unwrap();

        let hardware = create_test_hardware_info("test-client");
        let hardware_repo = HardwareRepository::new(db_arc);
        hardware_repo
            .save_hardware("test-client", &hardware, true)
            .await
            .unwrap();

        service.delete_client("test-client").await.unwrap();

        let hardware = hardware_repo.get_hardware("test-client").await.unwrap();
        assert!(hardware.is_none());
    }

    #[tokio::test]
    async fn test_import_empty_clients_list() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc);

        let result = service.import_clients(vec![]).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
