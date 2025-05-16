use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::entity::dictionary::Dictionary;
use serde_json;
use crate::db::Database;

/// Repository for dictionary operations
pub struct DictionaryRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl DictionaryRepository {
    /// Create a new dictionary repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "dictionary:".to_string(),
        }
    }
    
    /// Get dictionary key in database
    fn get_key(&self, id: &str) -> String {
        format!("{}{}", self.key_prefix, id)
    }
    
    /// Save a dictionary item to the database
    pub async fn save(&self, item: &Dictionary) -> CmdbResult<()> {
        let json = serde_json::to_vec(item)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize dictionary item: {}", e)))?;
        
        self.db.set(&self.get_key(&item.id), &json).await
    }
    
    /// Get a dictionary item by ID
    pub async fn get(&self, id: &str) -> CmdbResult<Option<Dictionary>> {
        let data = match self.db.get(&self.get_key(id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let item = serde_json::from_slice(&data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize dictionary item: {}", e)))?;
            
        Ok(Some(item))
    }
    
    /// Delete a dictionary item
    pub async fn delete(&self, id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(id)).await
    }
    
    /// List all dictionary items
    pub async fn list_all(&self) -> CmdbResult<Vec<Dictionary>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut items = Vec::with_capacity(values.len());
        
        for data in values {
            let item = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize dictionary item: {}", e)))?;
            items.push(item);
        }
        
        Ok(items)
    }

    /// List items by category
    pub async fn list_by_category(&self, category: &str) -> CmdbResult<Vec<Dictionary>> {
        let all = self.list_all().await?;
        Ok(all.into_iter().filter(|item| item.category == category).collect())
    }
}
