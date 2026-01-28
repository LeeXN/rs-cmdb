use crate::db::Database;
use common::error::{CmdbError, CmdbResult};
use common::models::Client;
use serde_json;
use std::sync::Arc;

/// Repository for client operations
pub struct ClientRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl ClientRepository {
    /// Create a new client repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "client:".to_string(),
        }
    }

    /// Get client key in database
    fn get_key(&self, client_id: &str) -> String {
        format!("{}{}", self.key_prefix, client_id)
    }

    /// Save a client to the database
    pub async fn save(&self, client: &Client) -> CmdbResult<()> {
        let client_json = serde_json::to_vec(client)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize client: {}", e)))?;

        self.db.set(&self.get_key(&client.id), &client_json).await
    }

    /// Get a client by ID
    pub async fn get(&self, client_id: &str) -> CmdbResult<Option<Client>> {
        let client_data = match self.db.get(&self.get_key(client_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };

        let client = serde_json::from_slice(&client_data).map_err(|e| {
            CmdbError::Serialization(format!("Failed to deserialize client: {}", e))
        })?;

        Ok(Some(client))
    }

    /// Check if a client exists
    pub async fn exists(&self, client_id: &str) -> CmdbResult<bool> {
        self.db.exists(&self.get_key(client_id)).await
    }

    /// Delete a client
    pub async fn delete(&self, client_id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(client_id)).await
    }

    /// List all clients
    pub async fn list_all(&self) -> CmdbResult<Vec<Client>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut clients = Vec::with_capacity(values.len());

        for data in values {
            let client = serde_json::from_slice(&data).map_err(|e| {
                CmdbError::Serialization(format!("Failed to deserialize client: {}", e))
            })?;
            clients.push(client);
        }

        Ok(clients)
    }

    /// Update client last seen timestamp
    pub async fn update_last_seen(&self, client_id: &str) -> CmdbResult<()> {
        let mut client = match self.get(client_id).await? {
            Some(c) => c,
            None => {
                return Err(CmdbError::NotFound(format!(
                    "Client {} not found",
                    client_id
                )));
            }
        };

        client.update_last_seen();
        self.save(&client).await
    }

    /// Count clients by rack ID
    pub async fn count_by_rack(&self, rack_id: &str) -> CmdbResult<usize> {
        let clients = self.list_all().await?;
        Ok(clients
            .iter()
            .filter(|c| c.rack.as_deref() == Some(rack_id))
            .count())
    }

    /// Count clients by project ID
    pub async fn count_by_project(&self, project_id: &str) -> CmdbResult<usize> {
        let clients = self.list_all().await?;
        Ok(clients
            .iter()
            .filter(|c| c.project_id.as_deref() == Some(project_id))
            .count())
    }

    /// Update owner to null for clients owned by a specific person
    pub async fn update_owner_to_null(&self, owner_id: &str) -> CmdbResult<()> {
        let clients = self.list_all().await?;
        for mut client in clients {
            if client.owner_id.as_deref() == Some(owner_id) {
                client.owner_id = None;
                self.save(&client).await?;
            }
        }
        Ok(())
    }

    /// Find client by serial number
    pub async fn find_by_serial(&self, serial_number: &str) -> CmdbResult<Option<Client>> {
        if serial_number.is_empty() || serial_number == "N/A" || serial_number == "Unknown" {
            return Ok(None);
        }
        let clients = self.list_all().await?;
        Ok(clients
            .into_iter()
            .find(|c| c.serial_number.as_deref() == Some(serial_number)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::models::{Client, ClientStatus, Environment};

    fn create_test_client(id: &str) -> Client {
        Client {
            id: id.to_string(),
            hostname: format!("test-{}", id),
            ip_address: format!("192.168.1.{}", id),
            os: Some("Linux".to_string()),
            kernel_version: Some("5.15.0".to_string()),
            serial_number: Some(format!("SN-{}", id)),
            sys_vendor: Some("Dell Inc.".to_string()),
            product_name: Some("PowerEdge R740".to_string()),
            last_seen: Some(chrono::Utc::now().to_rfc3339()),
            registered_at: Some(chrono::Utc::now().to_rfc3339()),
            comment: Some("Test client".to_string()),
            location: Some("DC1".to_string()),
            rack: Some("Rack01".to_string()),
            unit_position: Some("U10".to_string()),
            u_height: Some(2),
            project_id: Some("proj-001".to_string()),
            owner_id: Some("user-001".to_string()),
            status: Some(ClientStatus::Active),
            environment: Some(Environment::Prod),
            asset_tag: Some(format!("TAG-{}", id)),
            warranty_expiration: Some("2025-12-31".to_string()),
            supplier: Some("Dell".to_string()),
            power_consumption: Some(500),
        }
    }

    #[tokio::test]
    async fn test_save_client_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let client = create_test_client("client-001");
        let result = repo.save(&client).await;

        assert!(result.is_ok(), "Save should succeed with valid client data");
    }

    #[tokio::test]
    async fn test_get_client_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let client = create_test_client("client-002");
        repo.save(&client).await.unwrap();

        let result = repo.get("client-002").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(client)");
        let retrieved_client = retrieved.unwrap();
        assert_eq!(retrieved_client.id, "client-002");
        assert_eq!(retrieved_client.hostname, "test-client-002");
    }

    #[tokio::test]
    async fn test_get_client_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let result = repo.get("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(result.unwrap().is_none(), "Should return None");
    }

    #[tokio::test]
    async fn test_exists_when_client_exists_then_returns_true() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let client = create_test_client("client-003");
        repo.save(&client).await.unwrap();

        let result = repo.exists("client-003").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(result.unwrap(), "Should return true for existing client");
    }

    #[tokio::test]
    async fn test_exists_when_client_not_exists_then_returns_false() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let result = repo.exists("nonexistent").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(
            !result.unwrap(),
            "Should return false for non-existent client"
        );
    }

    #[tokio::test]
    async fn test_list_all_when_multiple_clients_then_returns_all() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        // Create multiple clients
        for i in 1..=5 {
            let client = create_test_client(&format!("client-00{}", i));
            repo.save(&client).await.unwrap();
        }

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        let clients = result.unwrap();
        assert_eq!(clients.len(), 5, "Should return all 5 clients");
    }

    #[tokio::test]
    async fn test_list_all_when_empty_then_returns_empty_vec() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        assert!(result.unwrap().is_empty(), "Should return empty vector");
    }

    #[tokio::test]
    async fn test_delete_client_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let client = create_test_client("client-004");
        repo.save(&client).await.unwrap();

        let delete_result = repo.delete("client-004").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        let get_result = repo.get("client-004").await;
        assert!(get_result.unwrap().is_none(), "Client should be deleted");
    }

    #[tokio::test]
    async fn test_delete_client_when_not_exists_then_still_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        // Deleting a non-existent client should not error (idempotent operation)
        let result = repo.delete("nonexistent").await;

        assert!(result.is_ok(), "Delete should be idempotent");
    }

    #[tokio::test]
    async fn test_update_last_seen_when_client_exists_then_updates() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let client = create_test_client("client-005");
        let original_last_seen = client.last_seen.clone().unwrap();
        repo.save(&client).await.unwrap();

        // Wait a bit to ensure timestamp would be different
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let result = repo.update_last_seen("client-005").await;

        assert!(result.is_ok(), "Update last seen should succeed");
        let updated = repo.get("client-005").await.unwrap().unwrap();
        assert_ne!(
            updated.last_seen.unwrap(),
            original_last_seen,
            "Last seen should be updated"
        );
    }

    #[tokio::test]
    async fn test_update_last_seen_when_client_not_exists_then_returns_error() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let result = repo.update_last_seen("nonexistent").await;

        assert!(
            result.is_err(),
            "Should return error for non-existent client"
        );
        match result.unwrap_err() {
            CmdbError::NotFound(_) => {}
            _ => panic!("Should return NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_count_by_rack_when_clients_exist_then_returns_count() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        // Create clients in different racks
        for i in 1..=3 {
            let mut client = create_test_client(&format!("client-0{}", i));
            client.rack = Some("Rack01".to_string());
            repo.save(&client).await.unwrap();
        }

        for i in 4..=6 {
            let mut client = create_test_client(&format!("client-0{}", i));
            client.rack = Some("Rack02".to_string());
            repo.save(&client).await.unwrap();
        }

        let rack01_count = repo.count_by_rack("Rack01").await.unwrap();
        let rack02_count = repo.count_by_rack("Rack02").await.unwrap();
        let empty_rack_count = repo.count_by_rack("Rack99").await.unwrap();

        assert_eq!(rack01_count, 3, "Should count 3 clients in Rack01");
        assert_eq!(rack02_count, 3, "Should count 3 clients in Rack02");
        assert_eq!(empty_rack_count, 0, "Should return 0 for empty rack");
    }

    #[tokio::test]
    async fn test_find_by_serial_when_exists_then_returns_client() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let mut client = create_test_client("client-006");
        client.serial_number = Some("SN-12345".to_string());
        repo.save(&client).await.unwrap();

        let result = repo.find_by_serial("SN-12345").await;

        assert!(result.is_ok(), "Find by serial should not error");
        let found = result.unwrap();
        assert!(found.is_some(), "Should find client");
        assert_eq!(found.unwrap().id, "client-006");
    }

    #[tokio::test]
    async fn test_find_by_serial_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let result = repo.find_by_serial("NONEXISTENT").await;

        assert!(result.is_ok(), "Find by serial should not error");
        assert!(result.unwrap().is_none(), "Should return None");
    }

    #[tokio::test]
    async fn test_find_by_serial_when_empty_or_special_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        // Test empty string
        let result = repo.find_by_serial("").await.unwrap();
        assert!(result.is_none(), "Empty string should return None");

        // Test "N/A"
        let result = repo.find_by_serial("N/A").await.unwrap();
        assert!(result.is_none(), "N/A should return None");

        // Test "Unknown"
        let result = repo.find_by_serial("Unknown").await.unwrap();
        assert!(result.is_none(), "Unknown should return None");
    }

    #[tokio::test]
    async fn test_save_multiple_clients_then_list_all_returns_all() {
        let db = setup_test_db().unwrap();
        let repo = ClientRepository::new(std::sync::Arc::new(db));

        let client_ids = vec!["client-001", "client-002", "client-003"];
        for id in &client_ids {
            let client = create_test_client(id);
            repo.save(&client).await.unwrap();
        }

        let result = repo.list_all().await.unwrap();

        assert_eq!(result.len(), 3, "Should return all saved clients");
        let returned_ids: Vec<&str> = result.iter().map(|c| c.id.as_str()).collect();
        for id in &client_ids {
            assert!(returned_ids.contains(id), "Should contain {}", id);
        }
    }
}
