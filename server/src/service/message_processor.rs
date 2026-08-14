use crate::queue::{Message, MessageQueue};
use crate::repository::command_repository::CommandRepository;
use crate::service::{client_service::ClientService, hardware_service::HardwareService};
use common::error::CmdbResult;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, instrument};

#[cfg(test)]
use common::command::{AuditAction, AuditLogEntry};

/// Service for processing messages from the queue
pub struct MessageProcessor {
    message_queue: Arc<dyn MessageQueue>,
    client_service: Arc<ClientService>,
    hardware_service: Arc<HardwareService>,
    command_repo: Arc<CommandRepository>,
}

impl MessageProcessor {
    /// Create a new message processor
    pub fn new(
        message_queue: Arc<dyn MessageQueue>,
        client_service: Arc<ClientService>,
        hardware_service: Arc<HardwareService>,
        command_repo: Arc<CommandRepository>,
    ) -> Self {
        Self {
            message_queue,
            client_service,
            hardware_service,
            command_repo,
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
                    error!(
                        "Error processing message (attempt {}/{}): {}",
                        retry_count, max_retries, e
                    );
                    last_error = Some(e);
                    // Simple exponential backoff: 1s, 2s, 3s...
                    time::sleep(Duration::from_secs(retry_count as u64)).await;
                }
            }
        }

        if let Some(e) = last_error {
            error!(
                "Failed to process message after {} attempts. Message dropped: {:?}",
                max_retries, message
            );
            // In a real system, we would send this to a Dead Letter Queue (DLQ)
            return Err(e);
        }

        Ok(())
    }

    /// Internal processing logic
    async fn process_message_internal(&self, message: Message) -> CmdbResult<()> {
        match message {
            Message::ClientHardwareInfo(hardware_info) => {
                info!(
                    "Processing hardware info from client: {}",
                    hardware_info.client_id
                );

                // Process hardware info
                self.hardware_service
                    .process_hardware_info(hardware_info)
                    .await?;
            }
            Message::PullRequest(request, client_id) => {
                info!("Processing pull request for client: {}", client_id);

                // In a real-world implementation, we would send a request to the client
                // This would typically involve some form of bidirectional communication
                // such as WebSockets or HTTP polling

                // For now, we just log the request
                info!(
                    "Pull request {} initiated for client {}",
                    request.request_id, client_id
                );
            }
            Message::PullResponse(response) => {
                info!(
                    "Processing pull response for request: {}",
                    response.request_id
                );

                // Process pull response
                self.hardware_service
                    .process_pull_response(response)
                    .await?;
            }
            Message::ClientHeartbeat(client_id) => {
                info!("Processing heartbeat from client: {}", client_id);

                // Update last seen timestamp
                self.client_service.update_last_seen(&client_id).await?;
            }
            Message::AuditLog(entry) => {
                info!("Recording audit log: {}", entry.action);
                self.command_repo.append_audit(&entry).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{CacheConfigs, CachedClientRepository};
    use crate::db::Database;
    use crate::queue::mock_queue::MockMessageQueue;
    use crate::repository::{
        client_repository::ClientRepository, command_repository::CommandRepository,
        component_repository::ComponentRepository, hardware_repository::HardwareRepository,
        rack_repository::RackRepository,
    };
    use crate::service::component_service::ComponentService;
    use crate::tests::fixtures::{create_client_hardware_info, create_test_client, setup_test_db};

    fn setup_processor(db: Arc<dyn Database>) -> (Arc<MockMessageQueue>, MessageProcessor) {
        let mq = Arc::new(MockMessageQueue::new());
        let mq_trait: Arc<dyn MessageQueue> = mq.clone();

        let client_repo_inner = Arc::new(ClientRepository::new(db.clone()));
        let cache_configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(
            client_repo_inner,
            &cache_configs,
        ));
        let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));
        let rack_repo = Arc::new(RackRepository::new(db.clone()));

        let client_service = Arc::new(ClientService::from_repositories(
            client_repo.clone(),
            hardware_repo.clone(),
            rack_repo,
        ));

        let component_repo = Arc::new(ComponentRepository::new(db.clone()));
        let component_svc = Arc::new(ComponentService::new(component_repo));
        let hw_svc = Arc::new(HardwareService::new(
            client_repo,
            hardware_repo,
            component_svc,
            mq_trait.clone(),
            None,
        ));

        let cmd_repo = Arc::new(CommandRepository::new(db));
        let processor = MessageProcessor::new(mq_trait, client_service, hw_svc, cmd_repo);
        (mq, processor)
    }

    #[tokio::test]
    async fn test_process_client_hardware_info() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);
        let (_mq, processor) = setup_processor(db.clone());

        let client_repo = ClientRepository::new(db.clone());
        let client = create_test_client("test-client");
        let client_id = client.id.clone();
        client_repo.save(&client).await.unwrap();

        let hw_info = create_client_hardware_info(&client_id);
        let msg = Message::ClientHardwareInfo(hw_info);

        let result = processor.process_message_internal(msg).await;
        assert!(result.is_ok());

        let hardware_repo = HardwareRepository::new(db);
        let saved = hardware_repo.get_hardware(&client_id).await.unwrap();
        assert!(saved.is_some());
    }

    #[tokio::test]
    async fn test_process_audit_log() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);
        let (_mq, processor) = setup_processor(db.clone());

        let entry = AuditLogEntry::new(AuditAction::CommandCreate, "admin", "Test audit log");
        let msg = Message::AuditLog(entry);
        let result = processor.process_message_internal(msg).await;
        assert!(result.is_ok());

        let cmd_repo = CommandRepository::new(db);
        let entries = cmd_repo.list_audit(1, 10).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, AuditAction::CommandCreate);
    }

    #[tokio::test]
    async fn test_process_client_heartbeat() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);
        let (_mq, processor) = setup_processor(db.clone());

        let client_repo = ClientRepository::new(db.clone());
        let client = create_test_client("test-heartbeat");
        let client_id = client.id.clone();
        client_repo.save(&client).await.unwrap();

        let initial = client_repo.get(&client_id).await.unwrap().unwrap();
        let initial_last_seen = initial.last_seen.clone();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let msg = Message::ClientHeartbeat(client_id.clone());
        let result = processor.process_message_internal(msg).await;
        assert!(result.is_ok());

        let updated = client_repo.get(&client_id).await.unwrap().unwrap();
        assert_ne!(initial_last_seen, updated.last_seen);
    }
}
