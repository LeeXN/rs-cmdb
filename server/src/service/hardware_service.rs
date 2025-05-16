use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::models::{PullResponse, ClientHardwareInfo};
use crate::repository::{client_repository::ClientRepository, hardware_repository::HardwareRepository};
use crate::service::component_service::ComponentService;
use crate::queue::MessageQueue;

/// Service for hardware operations
pub struct HardwareService {
    client_repo: Arc<ClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
    component_service: Arc<ComponentService>,
    #[allow(dead_code)]
    message_queue: Arc<dyn MessageQueue>,
}

impl HardwareService {
    /// Create a new hardware service
    pub fn new(
        client_repo: Arc<ClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
        component_service: Arc<ComponentService>,
        message_queue: Arc<dyn MessageQueue>,
    ) -> Self {
        Self {
            client_repo,
            hardware_repo,
            component_service,
            message_queue,
        }
    }
    
    /// Process hardware info from a client
    pub async fn process_hardware_info(&self, hardware_info: ClientHardwareInfo) -> CmdbResult<()> {
        let client_id = &hardware_info.client_id;
        
        // Check if client exists
        if let Ok(false) = self.client_repo.exists(client_id).await {
            return Err(CmdbError::NotFound(format!("Client {} not found", client_id)));
        }
        
        // Update last seen timestamp
        self.client_repo.update_last_seen(client_id).await?;
        
        // Save hardware info if available
        if let Some(hardware) = hardware_info.hardware {
            self.hardware_repo.save_hardware_with_timestamp(client_id, &hardware, true, Some(&hardware_info.collected_at)).await?;
            
            // Extract and update components
            if let Err(e) = self.component_service.process_hardware_info(client_id, &hardware).await {
                eprintln!("Error processing components for client {}: {}", client_id, e);
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
            return Err(CmdbError::Validation(format!("Invalid request ID format: {}", request_id)));
        }
        
        let client_id = parts[0];
        
        // Check if client exists
        if let Ok(false) = self.client_repo.exists(client_id).await {
            return Err(CmdbError::NotFound(format!("Client {} not found", client_id)));
        }
        
        // Update last seen timestamp
        self.client_repo.update_last_seen(client_id).await?;
        
        // Save hardware info if available and status is success
        if response.status == "success" {
            if let Some(hardware) = response.hardware {
                self.hardware_repo.save_hardware(client_id, &hardware, true).await?;
                
                // Extract and update components
                if let Err(e) = self.component_service.process_hardware_info(client_id, &hardware).await {
                    eprintln!("Error processing components for client {}: {}", client_id, e);
                }
            }
        }
        
        Ok(())
    }
} 