use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::entity::user::User;
use serde_json;
use crate::db::Database;

/// Repository for user operations
pub struct UserRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "user:".to_string(),
        }
    }
    
    /// Get user key in database
    fn get_key(&self, user_id: &str) -> String {
        format!("{}{}", self.key_prefix, user_id)
    }
    
    /// Save a user to the database
    pub async fn save(&self, user: &User) -> CmdbResult<()> {
        let user_json = serde_json::to_vec(user)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize user: {}", e)))?;
        
        self.db.set(&self.get_key(&user.id), &user_json).await
    }
    
    /// Get a user by ID
    pub async fn get(&self, user_id: &str) -> CmdbResult<Option<User>> {
        let user_data = match self.db.get(&self.get_key(user_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let user = serde_json::from_slice(&user_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize user: {}", e)))?;
            
        Ok(Some(user))
    }
    
    /// Find a user by username
    pub async fn find_by_username(&self, username: &str) -> CmdbResult<Option<User>> {
        // Since we don't have a secondary index, we iterate through all users
        // This is acceptable for a small number of users
        let users = self.list_all().await?;
        
        for user in users {
            if user.username == username {
                return Ok(Some(user));
            }
        }
        
        Ok(None)
    }
    
    /// Check if a user exists
    #[allow(dead_code)]
    pub async fn exists(&self, user_id: &str) -> CmdbResult<bool> {
        self.db.exists(&self.get_key(user_id)).await
    }
    
    /// Delete a user
    pub async fn delete(&self, user_id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(user_id)).await
    }
    
    /// List all users
    pub async fn list_all(&self) -> CmdbResult<Vec<User>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut users = Vec::with_capacity(values.len());
        
        for data in values {
            let user = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize user: {}", e)))?;
            users.push(user);
        }
        
        Ok(users)
    }
}
