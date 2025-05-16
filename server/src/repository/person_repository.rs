use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::models::Person;
use serde_json;
use crate::db::Database;

/// Repository for person operations
pub struct PersonRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl PersonRepository {
    /// Create a new person repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "person:".to_string(),
        }
    }
    
    /// Get person key in database
    fn get_key(&self, person_id: &str) -> String {
        format!("{}{}", self.key_prefix, person_id)
    }
    
    /// Save a person to the database
    pub async fn save(&self, person: &Person) -> CmdbResult<()> {
        let person_json = serde_json::to_vec(person)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize person: {}", e)))?;
        
        self.db.set(&self.get_key(&person.id), &person_json).await
    }
    
    /// Get a person by ID
    pub async fn get(&self, person_id: &str) -> CmdbResult<Option<Person>> {
        let person_data = match self.db.get(&self.get_key(person_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let person = serde_json::from_slice(&person_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize person: {}", e)))?;
            
        Ok(Some(person))
    }
    
    /// Check if a person exists
    pub async fn exists(&self, person_id: &str) -> CmdbResult<bool> {
        self.db.exists(&self.get_key(person_id)).await
    }
    
    /// Delete a person
    pub async fn delete(&self, person_id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(person_id)).await
    }
    
    /// List all persons
    pub async fn list_all(&self) -> CmdbResult<Vec<Person>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut persons = Vec::with_capacity(values.len());
        
        for data in values {
            let person = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize person: {}", e)))?;
            persons.push(person);
        }
        
        Ok(persons)
    }
}
