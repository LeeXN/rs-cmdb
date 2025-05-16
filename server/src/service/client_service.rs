use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::models::Client;
use crate::repository::{client_repository::ClientRepository, hardware_repository::HardwareRepository, rack_repository::RackRepository};
use tracing::{info, warn, instrument};

/// Service for client operations
pub struct ClientService {
    client_repo: Arc<ClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
    rack_repo: Arc<RackRepository>,
}

impl ClientService {
    /// Create a new client service
    pub fn new(client_repo: Arc<ClientRepository>, hardware_repo: Arc<HardwareRepository>, rack_repo: Arc<RackRepository>) -> Self {
        Self {
            client_repo,
            hardware_repo,
            rack_repo,
        }
    }
    
    /// Import clients from a list
    #[instrument(skip(self, clients))]
    pub async fn import_clients(&self, clients: Vec<Client>) -> CmdbResult<usize> {
        let mut count = 0;
        for client in clients {
            // Validate Rack Assignment
            if let Some(rack_id) = &client.rack {
                if !rack_id.is_empty() {
                    // Check if rack exists
                    let rack = self.rack_repo.get(rack_id).await?;
                    if rack.is_none() {
                        return Err(CmdbError::Validation(format!("Rack {} not found for client {}", rack_id, client.hostname)));
                    }
                    let rack = rack.unwrap();
                    
                    // Check if unit position is valid
                    if let Some(pos_str) = &client.unit_position {
                        if let Ok(pos) = pos_str.parse::<u32>() {
                            let height = client.u_height.unwrap_or(1);
                            if pos + height - 1 > rack.height_u {
                                return Err(CmdbError::Validation(format!("Client {} exceeds rack height", client.hostname)));
                            }
                            
                            // Check for overlap with existing clients
                            // This is expensive, we should optimize it in a real system
                            let all_clients = self.client_repo.list_all().await?;
                            for other in all_clients {
                                if other.id == client.id { continue; } // Skip self
                                if other.rack.as_ref() == Some(rack_id) {
                                    if let Some(other_pos_str) = &other.unit_position {
                                        if let Ok(other_pos) = other_pos_str.parse::<u32>() {
                                            let other_height = other.u_height.unwrap_or(1);
                                            // Check overlap
                                            let start1 = pos;
                                            let end1 = pos + height - 1;
                                            let start2 = other_pos;
                                            let end2 = other_pos + other_height - 1;
                                            
                                            if std::cmp::max(start1, start2) <= std::cmp::min(end1, end2) {
                                                return Err(CmdbError::Validation(format!("Rack position overlap: {} overlaps with {}", client.hostname, other.hostname)));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Save or Update
            if self.client_repo.exists(&client.id).await.unwrap_or(false) {
                self.client_repo.save(&client).await?;
            } else {
                // If ID is empty, generate one? Or assume import has IDs?
                // If import has no ID, we treat it as new?
                // For now, assume ID is present or we generate one if empty
                let mut new_client = client.clone();
                if new_client.id.is_empty() {
                    new_client.id = uuid::Uuid::new_v4().to_string();
                }
                self.client_repo.save(&new_client).await?;
            }
            count += 1;
        }
        Ok(count)
    }
    
    /// Register a new client
    /// 
    /// If client_id is provided, it will be used. Otherwise, a new UUID will be generated.
    #[instrument(skip(self, hostname, ip_address, sys_vendor, product_name, serial_number, os), fields(client_id))]
    pub async fn register_client(&self, hostname: &str, ip_address: &str, sys_vendor: &str, product_name: &str, serial_number: &str, os: &str, client_id: Option<String>) -> CmdbResult<Client> {
        // Create a new client with given or generated ID
        let mut client = Client::new(hostname.to_string(), ip_address.to_string());
        
        // Use provided client ID if available
        if let Some(id) = client_id {
            client.id = id;
        } else {
            // Try to find existing client by serial number if client_id is not provided
            if let Ok(Some(existing)) = self.client_repo.find_by_serial(serial_number).await {
                info!("Found existing client by serial number: {} -> {}", serial_number, existing.id);
                client.id = existing.id;
            }
        }
        
        // Check if client already exists
        if let Ok(true) = self.client_repo.exists(&client.id).await {
            // Update the client information
            if let Ok(Some(mut existing_client)) = self.client_repo.get(&client.id).await {
                existing_client.hostname = hostname.to_string();
                existing_client.ip_address = ip_address.to_string();
                existing_client.sys_vendor = Some(sys_vendor.to_string());
                existing_client.product_name = Some(product_name.to_string());
                existing_client.serial_number = Some(serial_number.to_string());
                existing_client.os = Some(os.to_string());
                existing_client.update_last_seen();
                
                self.client_repo.save(&existing_client).await?;
                info!("Client updated: {} ({})", existing_client.hostname, existing_client.id);
                return Ok(existing_client);
            }
        }
        
        // Save the new client
        self.client_repo.save(&client).await?;
        info!("New client registered: {} ({})", client.hostname, client.id);
        
        Ok(client)
    }
    
    /// Delete a client and associated hardware information
    /// Also attempts to stop the client service remotely
    #[instrument(skip(self))]
    pub async fn delete_client(&self, client_id: &str) -> CmdbResult<()> {
        // Get client information first
        let client = match self.client_repo.get(client_id).await? {
            Some(client) => client,
            None => return Err(CmdbError::NotFound(format!("Client {} not found", client_id))),
        };
        
        info!("Deleting client: {} ({})", client.hostname, client.ip_address);
        
        // Attempt to stop the client service remotely
        if let Err(e) = self.stop_client_service(&client).await {
            warn!("Failed to stop client service for {}: {}", client.hostname, e);
            // Continue with deletion even if we can't stop the service
        }
        
        // Delete hardware data
        match self.hardware_repo.delete_hardware(client_id).await {
            Ok(_) => info!("Hardware data deleted for client {}", client_id),
            Err(e) => warn!("Failed to delete hardware data for client {}: {}", client_id, e),
        }
        
        // Delete client from database
        self.client_repo.delete(client_id).await?;
        info!("Client {} deleted successfully from database", client_id);
        
        Ok(())
    }
    
    /// Attempt to stop the client service remotely
    #[instrument(skip(self, client), fields(hostname = %client.hostname, ip = %client.ip_address))]
    async fn stop_client_service(&self, client: &Client) -> CmdbResult<()> {
        info!("Attempting to stop service on client");
        
        // For Linux systems, try to stop systemd service via SSH
        if client.os.as_ref().map_or(false, |os| os.to_lowercase().contains("linux")) {
            return self.stop_linux_service(client).await;
        }
        
        // For Windows systems, try to stop service via PowerShell remoting
        if client.os.as_ref().map_or(false, |os| os.to_lowercase().contains("windows")) {
            return self.stop_windows_service(client).await;
        }
        
        warn!("Unsupported OS for remote service management: {:?}", client.os);
        Ok(())
    }
    
    /// Stop Linux service via SSH (if configured)
    async fn stop_linux_service(&self, client: &Client) -> CmdbResult<()> {
        // This is a simplified implementation
        // In production, you might want to use SSH keys or other authentication methods
        
        let commands = vec![
            "systemctl stop rs-cmdb-client",
            "systemctl disable rs-cmdb-client",
        ];
        
        for cmd in commands {
            info!("Executing on {}: {}", client.hostname, cmd);
            
            // Use tokio::process to execute SSH command
            let output = tokio::process::Command::new("ssh")
                .arg("-o")
                .arg("ConnectTimeout=10")
                .arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg(&client.ip_address)
                .arg(cmd)
                .output()
                .await;
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        info!("Command executed successfully on {}", client.hostname);
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        warn!("Command failed on {}: {}", client.hostname, stderr);
                    }
                }
                Err(e) => {
                    warn!("SSH connection failed to {}: {}", client.hostname, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Stop Windows service via PowerShell remoting (if configured)
    async fn stop_windows_service(&self, client: &Client) -> CmdbResult<()> {
        info!("Attempting to stop Windows service on {}", client.hostname);
        
        // This would require PowerShell remoting to be configured
        // For now, just log that we attempted it
        warn!("Windows service stopping not implemented yet for {}", client.hostname);
        
        Ok(())
    }
    
    /// Get a client by ID
    #[allow(dead_code)]
    pub async fn get_client(&self, client_id: &str) -> CmdbResult<Option<Client>> {
        self.client_repo.get(client_id).await
    }
    
    /// List all clients
    #[allow(dead_code)]
    pub async fn list_clients(&self) -> CmdbResult<Vec<Client>> {
        self.client_repo.list_all().await
    }
    
    /// Update client last seen timestamp
    pub async fn update_last_seen(&self, client_id: &str) -> CmdbResult<()> {
        self.client_repo.update_last_seen(client_id).await
    }
} 