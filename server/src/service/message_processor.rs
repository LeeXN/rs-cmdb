use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use common::error::CmdbResult;
use crate::queue::{MessageQueue, Message};
use crate::service::{
    client_service::ClientService,
    hardware_service::HardwareService,
};
use tracing::{info, error, instrument};

/// Service for processing messages from the queue
pub struct MessageProcessor {
    message_queue: Arc<dyn MessageQueue>,
    client_service: Arc<ClientService>,
    hardware_service: Arc<HardwareService>,
}

impl MessageProcessor {
    /// Create a new message processor
    pub fn new(
        message_queue: Arc<dyn MessageQueue>,
        client_service: Arc<ClientService>,
        hardware_service: Arc<HardwareService>,
    ) -> Self {
        Self {
            message_queue,
            client_service,
            hardware_service,
        }
    }
    
    /// Start processing messages in a loop
    pub async fn start(&self) -> CmdbResult<()> {
        info!("Starting message processor...");
        
        loop {
            // Process available messages
            self.process_messages().await?;
            
            // Sleep for a short time to avoid busy-waiting
            time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Process available messages from the queue
    async fn process_messages(&self) -> CmdbResult<()> {
        // Receive a message with timeout
        let timeout = Duration::from_secs(1);
        
        while let Ok(Some(message)) = self.message_queue.receive_message(timeout) {
            // Process the message
            if let Err(err) = self.process_message(message).await {
                error!("Error processing message: {}", err);
                // Continue processing other messages
            }
        }
        
        Ok(())
    }
    
    /// Process a single message
    #[instrument(skip(self, message))]
    async fn process_message(&self, message: Message) -> CmdbResult<()> {
        let max_retries = 3;
        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count < max_retries {
            // Clone message for retry since we might need it again
            match self.process_message_internal(message.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    retry_count += 1;
                    error!("Error processing message (attempt {}/{}): {}", retry_count, max_retries, e);
                    last_error = Some(e);
                    // Simple exponential backoff: 1s, 2s, 3s...
                    time::sleep(Duration::from_secs(retry_count as u64)).await;
                }
            }
        }
        
        if let Some(e) = last_error {
            error!("Failed to process message after {} attempts. Message dropped: {:?}", max_retries, message);
            // In a real system, we would send this to a Dead Letter Queue (DLQ)
            return Err(e);
        }
        
        Ok(())
    }

    /// Internal processing logic
    async fn process_message_internal(&self, message: Message) -> CmdbResult<()> {
        match message {
            Message::ClientRegistration(registration) => {
                info!("Processing client registration: {}", registration.id);
                
                // Register the client
                self.client_service.register_client(
                    &registration.hostname,
                    &registration.ip_address,
                    &registration.sys_vendor.unwrap_or_default(),
                    &registration.product_name.unwrap_or_default(),
                    &registration.serial_number.unwrap_or_default(),
                    &registration.os.unwrap_or_default(),
                    Some(registration.id.clone()),
                ).await?;
            },
            Message::ClientHardwareInfo(hardware_info) => {
                info!("Processing hardware info from client: {}", hardware_info.client_id);
                
                // Process hardware info
                self.hardware_service.process_hardware_info(hardware_info).await?;
            },
            Message::PullRequest(request, client_id) => {
                info!("Processing pull request for client: {}", client_id);
                
                // In a real-world implementation, we would send a request to the client
                // This would typically involve some form of bidirectional communication
                // such as WebSockets or HTTP polling
                
                // For now, we just log the request
                info!("Pull request {} initiated for client {}", request.request_id, client_id);
            },
            Message::PullResponse(response) => {
                info!("Processing pull response for request: {}", response.request_id);
                
                // Process pull response
                self.hardware_service.process_pull_response(response).await?;
            },
            Message::ClientHeartbeat(client_id) => {
                info!("Processing heartbeat from client: {}", client_id);
                
                // Update last seen timestamp
                self.client_service.update_last_seen(&client_id).await?;
            },
        }
        
        Ok(())
    }
} 