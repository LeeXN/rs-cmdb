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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub async fn get_hardware(&self, client_id: &str) -> CmdbResult<Option<Hardware>> {
        self.hardware_repo.get_hardware(client_id).await
    }

    /// Get all clients with their hardware
    #[instrument(skip(self))]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub async fn list_by_rack(&self, rack_id: &str) -> CmdbResult<Vec<Client>> {
        let all_clients = self.list_all().await?;
        Ok(all_clients
            .into_iter()
            .filter(|c| c.rack.as_deref() == Some(rack_id))
            .collect())
    }

    /// Check if a hostname exists
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn hostname_exists(&self, hostname: &str) -> CmdbResult<bool> {
        let all_clients = self.list_all().await?;
        Ok(all_clients.iter().any(|c| c.hostname == hostname))
    }

    /// Get clients by status
    #[instrument(skip(self))]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheConfigs;
    use crate::db::Database;
    use crate::repository::client_repository::ClientRepository;
    use crate::tests::fixtures::{create_test_hardware_info, setup_test_db};

    #[tokio::test]
    async fn test_get_and_save_client() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(inner, &configs));
        let hardware_repo = Arc::new(HardwareRepository::new(db));
        let dao = ClientDao::new(client_repo, hardware_repo);

        let client = Client::new("test-host".into(), "192.168.1.100".into());
        dao.save(&client).await.unwrap();

        let retrieved = dao.get(&client.id).await.unwrap().unwrap();
        assert_eq!(retrieved.hostname, "test-host");
        assert_eq!(retrieved.ip_address, "192.168.1.100");
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(inner, &configs));
        let hardware_repo = Arc::new(HardwareRepository::new(db));
        let dao = ClientDao::new(client_repo, hardware_repo);

        let c1 = Client::new("host-1".into(), "10.0.0.1".into());
        let c2 = Client::new("host-2".into(), "10.0.0.2".into());
        dao.save(&c1).await.unwrap();
        dao.save(&c2).await.unwrap();

        let all = dao.list_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(inner, &configs));
        let hardware_repo = Arc::new(HardwareRepository::new(db));
        let dao = ClientDao::new(client_repo, hardware_repo);

        let client = Client::new("to-delete".into(), "10.0.0.1".into());
        dao.save(&client).await.unwrap();
        dao.delete(&client.id).await.unwrap();

        assert!(dao.get(&client.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_hostname_exists() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(inner, &configs));
        let hardware_repo = Arc::new(HardwareRepository::new(db));
        let dao = ClientDao::new(client_repo, hardware_repo);

        let client = Client::new("unique-host".into(), "10.0.0.1".into());
        dao.save(&client).await.unwrap();

        assert!(dao.hostname_exists("unique-host").await.unwrap());
        assert!(!dao.hostname_exists("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_by_rack() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(inner, &configs));
        let hardware_repo = Arc::new(HardwareRepository::new(db));
        let dao = ClientDao::new(client_repo, hardware_repo);

        let mut c1 = Client::new("rack-a-host".into(), "10.0.0.1".into());
        c1.rack = Some("rack-a".into());
        dao.save(&c1).await.unwrap();

        let mut c2 = Client::new("rack-b-host".into(), "10.0.0.2".into());
        c2.rack = Some("rack-b".into());
        dao.save(&c2).await.unwrap();

        let ra = dao.list_by_rack("rack-a").await.unwrap();
        assert_eq!(ra.len(), 1);
        assert_eq!(ra[0].hostname, "rack-a-host");

        let rb = dao.list_by_rack("rack-b").await.unwrap();
        assert_eq!(rb.len(), 1);
    }

    #[tokio::test]
    async fn test_get_hardware() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(inner, &configs));
        let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));
        let dao = ClientDao::new(client_repo, hardware_repo.clone());

        let client = Client::new("hw-host".into(), "10.0.0.1".into());
        dao.save(&client).await.unwrap();

        let hardware = create_test_hardware_info(&client.id);
        hardware_repo
            .save_hardware(&client.id, &hardware, false)
            .await
            .unwrap();

        let retrieved = dao.get_hardware(&client.id).await.unwrap().unwrap();
        assert_eq!(
            retrieved.system.unwrap().serial_number,
            hardware.system.unwrap().serial_number
        );
    }
}
