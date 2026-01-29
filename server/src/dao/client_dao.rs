//! Client Data Access Object
//!
//! Encapsulates all data access operations related to clients,
//! including queries that span multiple repositories.

use crate::cache::CachedClientRepository;
use crate::repository::hardware_repository::HardwareRepository;
use common::entity::hardware::Hardware;
use common::error::CmdbResult;
use common::models::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;

/// Data Access Object for Client operations
pub struct ClientDao {
    client_repo: Arc<CachedClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
}

impl ClientDao {
    /// Create a new ClientDao
    pub fn new(
        client_repo: Arc<CachedClientRepository>,
        hardware_repo: Arc<HardwareRepository>,
    ) -> Self {
        Self {
            client_repo,
            hardware_repo,
        }
    }

    /// Get a client by ID
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> CmdbResult<Option<Client>> {
        self.client_repo.get(id).await
    }

    /// Get all clients
    #[instrument(skip(self))]
    pub async fn list_all(&self) -> CmdbResult<Vec<Client>> {
        self.client_repo.list_all().await
    }

    /// Save a client
    #[instrument(skip(self, client))]
    pub async fn save(&self, client: &Client) -> CmdbResult<()> {
        self.client_repo.save(client).await
    }

    /// Delete a client
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> CmdbResult<()> {
        self.client_repo.delete(id).await
    }

    /// Get hardware for a client
    #[instrument(skip(self))]
    pub async fn get_hardware(&self, client_id: &str) -> CmdbResult<Option<Hardware>> {
        self.hardware_repo.get_hardware(client_id).await
    }

    /// Get all clients with their hardware
    #[instrument(skip(self))]
    pub async fn list_with_hardware(&self) -> CmdbResult<HashMap<String, (Client, Hardware)>> {
        let clients = self.list_all().await?;
        let mut result = HashMap::new();

        for client in clients {
            if let Ok(Some(hardware)) = self.get_hardware(&client.id).await {
                result.insert(client.id.clone(), (client, hardware));
            }
        }

        Ok(result)
    }

    /// Get clients in a rack
    #[instrument(skip(self))]
    pub async fn list_by_rack(&self, rack_id: &str) -> CmdbResult<Vec<Client>> {
        let all_clients = self.list_all().await?;
        Ok(all_clients
            .into_iter()
            .filter(|c| c.rack.as_deref() == Some(rack_id))
            .collect())
    }

    /// Check if a hostname exists
    #[instrument(skip(self))]
    pub async fn hostname_exists(&self, hostname: &str) -> CmdbResult<bool> {
        let all_clients = self.list_all().await?;
        Ok(all_clients.iter().any(|c| c.hostname == hostname))
    }

    /// Get clients by status
    #[instrument(skip(self))]
    pub async fn list_by_status(&self, status: &str) -> CmdbResult<Vec<Client>> {
        let all_clients = self.list_all().await?;
        Ok(all_clients
            .into_iter()
            .filter(|c| {
                c.status
                    .as_ref()
                    .is_some_and(|s| format!("{:?}", s) == status)
            })
            .collect())
    }

    /// Get online clients (last seen within 5 minutes)
    #[instrument(skip(self))]
    pub async fn list_online(&self) -> CmdbResult<Vec<Client>> {
        let all_clients = self.list_all().await?;
        let now = chrono::Utc::now();

        Ok(all_clients
            .into_iter()
            .filter(|c| {
                c.last_seen
                    .as_ref()
                    .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                    .map(|dt| {
                        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                        duration.num_minutes() <= 5
                    })
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Get offline clients (last seen more than 5 minutes ago)
    #[instrument(skip(self))]
    pub async fn list_offline(&self) -> CmdbResult<Vec<Client>> {
        let all_clients = self.list_all().await?;
        let now = chrono::Utc::now();

        Ok(all_clients
            .into_iter()
            .filter(|c| {
                c.last_seen
                    .as_ref()
                    .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                    .map(|dt| {
                        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                        duration.num_minutes() > 5
                    })
                    .unwrap_or(true)
            })
            .collect())
    }
}
