use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::models::Client;
use serde_json;
use crate::db::Database;

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
        
        let client = serde_json::from_slice(&client_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize client: {}", e)))?;
            
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
            let client = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize client: {}", e)))?;
            clients.push(client);
        }
        
        Ok(clients)
    }
    
    /// Update client last seen timestamp
    pub async fn update_last_seen(&self, client_id: &str) -> CmdbResult<()> {
        let mut client = match self.get(client_id).await? {
            Some(c) => c,
            None => return Err(CmdbError::NotFound(format!("Client {} not found", client_id))),
        };
        
        client.update_last_seen();
        self.save(&client).await
    }

    /// Count clients by rack ID
    pub async fn count_by_rack(&self, rack_id: &str) -> CmdbResult<usize> {
        let clients = self.list_all().await?;
        Ok(clients.iter().filter(|c| c.rack.as_deref() == Some(rack_id)).count())
    }

    /// Count clients by project ID
    pub async fn count_by_project(&self, project_id: &str) -> CmdbResult<usize> {
        let clients = self.list_all().await?;
        Ok(clients.iter().filter(|c| c.project_id.as_deref() == Some(project_id)).count())
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
        Ok(clients.into_iter().find(|c| c.serial_number.as_deref() == Some(serial_number)))
    }
} 