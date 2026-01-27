use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::models::Rack;
use serde_json;
use crate::db::Database;

/// Repository for rack operations
pub struct RackRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl RackRepository {
    /// Create a new rack repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "rack:".to_string(),
        }
    }
    
    /// Get rack key in database
    fn get_key(&self, rack_id: &str) -> String {
        format!("{}{}", self.key_prefix, rack_id)
    }
    
    /// Save a rack to the database
    pub async fn save(&self, rack: &Rack) -> CmdbResult<()> {
        let rack_json = serde_json::to_vec(rack)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize rack: {}", e)))?;
        
        self.db.set(&self.get_key(&rack.id), &rack_json).await
    }
    
    /// Get a rack by ID
    pub async fn get(&self, rack_id: &str) -> CmdbResult<Option<Rack>> {
        let rack_data = match self.db.get(&self.get_key(rack_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let rack = serde_json::from_slice(&rack_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize rack: {}", e)))?;
            
        Ok(Some(rack))
    }
    
    /// Check if a rack exists
    pub async fn exists(&self, rack_id: &str) -> CmdbResult<bool> {
        self.db.exists(&self.get_key(rack_id)).await
    }
    
    /// Delete a rack
    pub async fn delete(&self, rack_id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(rack_id)).await
    }
    
    /// List all racks
    pub async fn list_all(&self) -> CmdbResult<Vec<Rack>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut racks = Vec::with_capacity(values.len());
        
        for data in values {
            let rack = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize rack: {}", e)))?;
            racks.push(rack);
        }
        
        Ok(racks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::models::{Rack, RackQuery};

    fn create_test_rack(id: &str, name: &str) -> Rack {
        Rack {
            id: id.to_string(),
            name: name.to_string(),
            location: Some(format!("Location-{}", id)),
            height_u: 42,
            power_limit: Some(1000),
            description: Some(format!("Test rack {}", id)),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_save_rack_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let rack = create_test_rack("rack-001", "Main Rack");
        let result = repo.save(&rack).await;

        assert!(result.is_ok(), "Save should succeed");
    }

    #[tokio::test]
    async fn test_get_rack_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let rack = create_test_rack("rack-002", "Server Rack");
        repo.save(&rack).await.unwrap();

        let result = repo.get("rack-002").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(rack)");
        assert_eq!(retrieved.unwrap().name, "Server Rack");
    }

    #[tokio::test]
    async fn test_get_rack_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let result = repo.get("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(result.unwrap().is_none(), "Should return None for non-existent rack");
    }

    #[tokio::test]
    async fn test_exists_when_rack_exists_then_returns_true() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let rack = create_test_rack("rack-003", "Storage Rack");
        repo.save(&rack).await.unwrap();

        let result = repo.exists("rack-003").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(result.unwrap(), "Should return true for existing rack");
    }

    #[tokio::test]
    async fn test_exists_when_rack_not_exists_then_returns_false() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let result = repo.exists("nonexistent").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(!result.unwrap(), "Should return false for non-existent rack");
    }

    #[tokio::test]
    async fn test_delete_rack_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let rack = create_test_rack("rack-004", "Backup Rack");
        repo.save(&rack).await.unwrap();

        let delete_result = repo.delete("rack-004").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        let get_result = repo.get("rack-004").await;
        assert!(get_result.unwrap().is_none(), "Rack should be deleted");
    }

    #[tokio::test]
    async fn test_delete_rack_when_not_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let result = repo.delete("nonexistent").await;

        assert!(result.is_ok(), "Delete should be idempotent");
    }

    #[tokio::test]
    async fn test_list_all_when_multiple_racks_then_returns_all() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let rack1 = create_test_rack("rack-005", "Rack A");
        let rack2 = create_test_rack("rack-006", "Rack B");
        let rack3 = create_test_rack("rack-007", "Rack C");

        repo.save(&rack1).await.unwrap();
        repo.save(&rack2).await.unwrap();
        repo.save(&rack3).await.unwrap();

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        let racks = result.unwrap();
        assert_eq!(racks.len(), 3, "Should return all racks");
    }

    #[tokio::test]
    async fn test_list_all_when_empty_then_returns_empty_vec() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        assert!(result.unwrap().is_empty(), "Should return empty vec for empty db");
    }

    #[tokio::test]
    async fn test_save_rack_with_duplicate_id_then_replaces() {
        let db = setup_test_db().unwrap();
        let repo = RackRepository::new(std::sync::Arc::new(db));

        let rack1 = create_test_rack("rack-008", "Original Rack");
        let mut rack2 = create_test_rack("rack-008", "Updated Rack");
        rack2.height_u = 48;

        repo.save(&rack1).await.unwrap();
        repo.save(&rack2).await.unwrap();

        let result = repo.get("rack-008").await;
        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return the rack");
        assert_eq!(retrieved.unwrap().height_u, 48, "Should have updated height");
    }
}
