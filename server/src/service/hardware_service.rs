use crate::queue::MessageQueue;
use crate::repository::{
    client_repository::ClientRepository, hardware_repository::HardwareRepository,
};
use crate::service::component_service::ComponentService;
use common::error::{CmdbError, CmdbResult};
use common::models::{ClientHardwareInfo, PullResponse};
use std::sync::Arc;

#[cfg(test)]
use crate::tests::fixtures::*;

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
    use crate::repository::{
        client_repository::ClientRepository, component_repository::ComponentRepository,
        hardware_repository::HardwareRepository,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_hardware_service_creation() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));

        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let _service = HardwareService::new(
            client_repo,
            hardware_repo,
            Arc::new(ComponentService::new(Arc::new(ComponentRepository::new(
                Arc::clone(&db_arc),
            )))),
            mock_queue,
        );
    }

    #[tokio::test]
    async fn test_process_hardware_info_with_valid_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service = HardwareService::new(
            client_repo.clone(),
            hardware_repo,
            component_service,
            mock_queue,
        );

        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let hardware_info = create_client_hardware_info("client-1");
        let result = service.process_hardware_info(hardware_info).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_hardware_info_with_nonexistent_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service =
            HardwareService::new(client_repo, hardware_repo, component_service, mock_queue);

        let hardware_info = create_client_hardware_info("nonexistent");
        let result = service.process_hardware_info(hardware_info).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_hardware_info_updates_last_seen() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service = HardwareService::new(
            client_repo.clone(),
            hardware_repo,
            component_service,
            mock_queue,
        );

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
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service = HardwareService::new(
            client_repo.clone(),
            hardware_repo.clone(),
            component_service,
            mock_queue,
        );

        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let hardware_info = create_client_hardware_info("client-1");
        service.process_hardware_info(hardware_info).await.unwrap();

        let hardware = hardware_repo.get_hardware("client-1").await.unwrap();
        assert!(hardware.is_some());
    }

    #[tokio::test]
    async fn test_process_hardware_info_without_hardware() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service = HardwareService::new(
            client_repo.clone(),
            hardware_repo.clone(),
            component_service,
            mock_queue,
        );

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
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service = HardwareService::new(
            client_repo.clone(),
            hardware_repo.clone(),
            component_service,
            mock_queue,
        );

        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let pull_response = create_pull_response("client-1", "success");
        let result = service.process_pull_response(pull_response).await;

        assert!(result.is_ok());

        let hardware = hardware_repo.get_hardware("client-1").await.unwrap();
        assert!(hardware.is_some());
    }

    #[tokio::test]
    async fn test_process_pull_response_with_error() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service = HardwareService::new(
            client_repo.clone(),
            hardware_repo.clone(),
            component_service,
            mock_queue,
        );

        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let pull_response = create_pull_response("client-1", "error");
        let result = service.process_pull_response(pull_response).await;

        assert!(result.is_ok());

        let hardware = hardware_repo.get_hardware("client-1").await.unwrap();
        assert!(hardware.is_none());
    }

    #[tokio::test]
    async fn test_process_pull_response_with_invalid_request_id() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service =
            HardwareService::new(client_repo, hardware_repo, component_service, mock_queue);

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
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let hardware_repo = Arc::new(HardwareRepository::new(Arc::clone(&db_arc)));
        let component_service = Arc::new(ComponentService::new(Arc::new(
            ComponentRepository::new(Arc::clone(&db_arc)),
        )));
        let mock_queue = Arc::new(crate::queue::mock_queue::MockMessageQueue::new());

        let service =
            HardwareService::new(client_repo, hardware_repo, component_service, mock_queue);

        let pull_response = create_pull_response("nonexistent", "success");
        let result = service.process_pull_response(pull_response).await;

        assert!(result.is_err());
    }
}
