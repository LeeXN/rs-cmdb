//! Client service for client business logic
//!
//! This service uses DAO layer for data access, reducing coupling
//! and simplifying business logic.

use crate::dao::{ClientDao, RackDao};
use crate::queue::{Message, MessageQueue};
use crate::repository::hardware_repository::HardwareRepository;
use crate::service::auth_service::{
    client_token_matches_id, generate_client_token, hash_token, verify_token,
};
use crate::validation::validate_ip_address;
use common::command::{AuditAction, AuditLogEntry};
use common::error::{CmdbError, CmdbResult};
use common::models::Client;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, instrument, warn};

const REMOTE_SERVICE_STOP_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(test)]
use crate::tests::fixtures::*;

/// Service for client operations
pub struct ClientService {
    client_dao: Arc<ClientDao>,
    rack_dao: Arc<RackDao>,
    hardware_repo: Arc<HardwareRepository>,
    message_queue: Option<Arc<dyn MessageQueue>>,
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
            message_queue: None,
        }
    }

    #[allow(dead_code)]
    /// Create a new client service from repositories (backward compatibility)
    pub fn from_repositories(
        client_repo: Arc<crate::cache::CachedClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
        rack_repo: Arc<crate::repository::rack_repository::RackRepository>,
    ) -> Self {
        Self {
            client_dao: Arc::new(ClientDao::new(client_repo.clone(), hardware_repo.clone())),
            rack_dao: Arc::new(RackDao::new(rack_repo, client_repo)),
            hardware_repo,
            message_queue: None,
        }
    }

    pub fn with_queue(mut self, message_queue: Arc<dyn MessageQueue>) -> Self {
        self.message_queue = Some(message_queue);
        self
    }

    #[allow(dead_code)]
    fn send_audit(&self, action: AuditAction, operator: &str, detail: &str) {
        if let Some(ref queue) = self.message_queue {
            let entry = AuditLogEntry::new(action, operator, detail);
            let _ = queue.send_message(Message::AuditLog(entry));
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
        primary_ip: Option<String>,
    ) -> CmdbResult<(Client, String)> {
        // Create a new client with given or generated ID
        let mut client = Client::new(hostname.to_string(), ip_address.to_string());
        client.primary_ip = primary_ip.clone();
        client.sys_vendor = Some(sys_vendor.to_string());
        client.product_name = Some(product_name.to_string());
        client.serial_number = Some(serial_number.to_string());
        client.os = Some(os.to_string());

        // Treat an empty or whitespace-only ID as absent. This prevents any
        // caller from persisting the reserved database key `client:`.
        let client_id = client_id
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty());

        // Use provided client ID if available
        if let Some(id) = client_id {
            client.id = id;
        } else {
            // Try to find existing client by serial number if client_id is not provided
            // Note: This uses the underlying repo since find_by_serial is repo-specific
            // In a full refactor, this would also be in the DAO
            if let Ok(Some(existing)) = self.client_dao.get_by_serial(serial_number).await {
                info!(
                    "Found existing client by serial number: {} -> {}",
                    serial_number, existing.id
                );
                client.id = existing.id;
            }
        }

        // Generate a deterministic token bound to this client_id via HMAC.
        let plain_token = generate_client_token(&client.id);

        // Hash the token before storing
        let hashed_token = hash_token(&plain_token)?;

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
                // Only update primary_ip if explicitly provided by the agent
                if let Some(ref pip) = primary_ip {
                    existing_client.primary_ip = Some(pip.clone());
                }
                existing_client.update_last_seen();
                // Store hashed token
                existing_client.agent_token = Some(hashed_token);

                self.client_dao.save(&existing_client).await?;
                info!(
                    "Client updated: {} ({})",
                    existing_client.hostname, existing_client.id
                );
                // Return client without token in the record (token only in the wrapper response)
                existing_client.agent_token = None;
                return Ok((existing_client, plain_token));
            }
        }

        // Save the new client with the hashed token
        client.agent_token = Some(hashed_token);
        self.client_dao.save(&client).await?;
        info!("New client registered: {} ({})", client.hostname, client.id);

        // Strip token from returned record
        client.agent_token = None;
        Ok((client, plain_token))
    }

    /// Delete a client and associated hardware information
    /// Also attempts to stop the client service remotely
    #[instrument(skip(self))]
    pub async fn delete_client(&self, client_id: &str) -> CmdbResult<()> {
        // Get client information first
        let client = self
            .client_dao
            .get(client_id)
            .await?
            .ok_or_else(|| CmdbError::NotFound(format!("Client {} not found", client_id)))?;

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

        let config = crate::config::get_config();
        let known_hosts_file = config.ssh_known_hosts_file.as_deref();

        let command = "systemctl stop rs-cmdb-client; systemctl disable rs-cmdb-client";
        info!("Executing on {}: {}", client.hostname, command);

        let mut ssh_cmd = tokio::process::Command::new("ssh");
        ssh_cmd
            .kill_on_drop(true)
            .arg("-o")
            .arg("ConnectTimeout=3")
            .arg("-o")
            .arg("ConnectionAttempts=1")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("StrictHostKeyChecking=yes");

        if let Some(known_hosts) = known_hosts_file {
            ssh_cmd
                .arg("-o")
                .arg(format!("UserKnownHostsFile={}", known_hosts));
        }

        // Use validated IP address. Keep remote cleanup best-effort and tightly
        // bounded so an offline client cannot make the DELETE API time out.
        ssh_cmd.arg(&validated_ip).arg(command);

        let output = tokio::time::timeout(REMOTE_SERVICE_STOP_TIMEOUT, ssh_cmd.output())
            .await
            .map_err(|_| {
                CmdbError::Internal(format!(
                    "Timed out stopping service on {} after {} seconds",
                    client.hostname,
                    REMOTE_SERVICE_STOP_TIMEOUT.as_secs()
                ))
            })?;

        match output {
            Ok(result) if result.status.success() => {
                info!("Service stopped successfully on {}", client.hostname);
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
                warn!("Command failed on {}: {}", client.hostname, stderr);

                if stderr.contains("Host key verification failed") {
                    return Err(CmdbError::Validation(format!(
                        "SSH host key verification failed for {}. Please add the host to your known_hosts file.",
                        client.hostname
                    )));
                }

                return Err(CmdbError::Internal(format!(
                    "Failed to stop service on {}: {}",
                    client.hostname, stderr
                )));
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

    /// Find a client by machine serial number.
    pub async fn get_by_serial(&self, serial: &str) -> CmdbResult<Option<Client>> {
        self.client_dao.get_by_serial(serial).await
    }

    /// Identify the canonical record that originally received a valid token.
    /// This supports releases that persisted the credential as
    /// `agent_token_unknown` before they persisted the server-returned ID.
    pub async fn get_by_issued_agent_token(&self, token: &str) -> CmdbResult<Option<Client>> {
        if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Ok(None);
        }
        for client in self.client_dao.list_all().await? {
            if client_token_matches_id(token, &client.id)
                && client
                    .agent_token
                    .as_deref()
                    .is_some_and(|hash| verify_token(token, hash).unwrap_or(false))
            {
                return Ok(Some(client));
            }
        }
        Ok(None)
    }

    /// Re-issue the credential for an existing Agent. This is intentionally
    /// exposed only through an admin-protected API; normal registration must
    /// continue proving possession of the current token.
    pub async fn recover_agent_token(&self, client_id: &str) -> CmdbResult<String> {
        let mut client = self
            .client_dao
            .get(client_id)
            .await?
            .ok_or_else(|| CmdbError::NotFound(format!("Client {} not found", client_id)))?;
        let token = generate_client_token(client_id);
        client.agent_token = Some(hash_token(&token)?);
        self.client_dao.save(&client).await?;
        Ok(token)
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
    use crate::cache::{CacheConfigs, CachedClientRepository};
    use crate::repository::{
        client_repository::ClientRepository, hardware_repository::HardwareRepository,
        rack_repository::RackRepository,
    };
    use std::sync::Arc;

    fn create_service(db: Arc<dyn crate::db::Database>) -> ClientService {
        let client_repo_inner = Arc::new(ClientRepository::new(db.clone()));
        let cache_configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(
            client_repo_inner.clone(),
            &cache_configs,
        ));

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
                None,
            )
            .await;

        assert!(client.is_ok());
        let (client, _token) = client.unwrap();
        assert_eq!(client.hostname, "test-host");
        assert_eq!(client.ip_address, "192.168.1.1");
        assert_eq!(client.sys_vendor.as_deref(), Some("Dell"));
        assert_eq!(client.product_name.as_deref(), Some("PowerEdge"));
        assert_eq!(client.serial_number.as_deref(), Some("SN12345"));
        assert_eq!(client.os.as_deref(), Some("Linux"));
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
                None,
            )
            .await;

        assert!(client.is_ok());
        assert_eq!(client.unwrap().0.id, custom_id);
    }

    #[tokio::test]
    async fn test_register_client_treats_blank_id_as_absent() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let service = create_service(db_arc.clone());

        let (client, _token) = service
            .register_client(
                "blank-id-host",
                "192.168.1.2",
                "Dell",
                "PowerEdge",
                "SN-BLANK-ID",
                "Linux",
                Some("   ".to_string()),
                None,
            )
            .await
            .unwrap();

        assert!(!client.id.is_empty());
        assert!(
            ClientRepository::new(db_arc)
                .get("")
                .await
                .unwrap()
                .is_none()
        );
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
                None,
            )
            .await;

        assert!(result.is_ok());
        let (client, _token) = result.unwrap();
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
