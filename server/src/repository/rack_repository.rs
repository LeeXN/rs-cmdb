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
