use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::entity::hardware::Hardware;
use serde_json;
use crate::db::Database;

/// Repository for hardware information operations
pub struct HardwareRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl HardwareRepository {
    /// Create a new hardware repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "hardware:".to_string(),
        }
    }
    
    /// Get hardware key in database
    fn get_key(&self, client_id: &str) -> String {
        format!("{}{}", self.key_prefix, client_id)
    }
    
    /// Get history key in database
    fn get_history_key(&self, client_id: &str, timestamp: &str) -> String {
        format!("{}{}:history:{}", self.key_prefix, client_id, timestamp)
    }
    
    /// Save hardware information to the database
    pub async fn save_hardware(&self, client_id: &str, hardware: &Hardware, save_history: bool) -> CmdbResult<()> {
        let hardware_json = serde_json::to_vec(hardware)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize hardware: {}", e)))?;
        
        // Check if hardware has changed (only if we need to save history)
        let hardware_changed = if save_history {
            match self.get_hardware(client_id).await? {
                Some(existing_hardware) => {
                    // Compare the hardware configurations
                    !self.hardware_equals(&existing_hardware, hardware)
                }
                None => {
                    // No existing hardware, this is the first time
                    true
                }
            }
        } else {
            false
        };
        
        // Save current hardware data
        self.db.set(&self.get_key(client_id), &hardware_json).await?;
        
        // Only save to history if hardware has actually changed
        if save_history && hardware_changed {
            let timestamp = chrono::Utc::now().timestamp().to_string();
            self.db.set(&self.get_history_key(client_id, &timestamp), &hardware_json).await?;
        }
        
        Ok(())
    }
    
    /// Save hardware information to the database with custom timestamp
    pub async fn save_hardware_with_timestamp(&self, client_id: &str, hardware: &Hardware, save_history: bool, custom_timestamp: Option<&str>) -> CmdbResult<()> {
        let hardware_json = serde_json::to_vec(hardware)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize hardware: {}", e)))?;
        
        // Check if hardware has changed (only if we need to save history)
        let hardware_changed = if save_history {
            match self.get_hardware(client_id).await? {
                Some(existing_hardware) => {
                    // Compare the hardware configurations
                    !self.hardware_equals(&existing_hardware, hardware)
                }
                None => {
                    // No existing hardware, this is the first time
                    true
                }
            }
        } else {
            false
        };
        
        // Save current hardware data
        self.db.set(&self.get_key(client_id), &hardware_json).await?;
        
        // Only save to history if hardware has actually changed
        if save_history && hardware_changed {
            let timestamp = if let Some(ts) = custom_timestamp {
                // Try to parse the timestamp to validate it and convert to Unix timestamp
                if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(ts) {
                    datetime.timestamp().to_string()
                } else {
                    // If parsing fails, use current timestamp
                    chrono::Utc::now().timestamp().to_string()
                }
            } else {
                chrono::Utc::now().timestamp().to_string()
            };
            self.db.set(&self.get_history_key(client_id, &timestamp), &hardware_json).await?;
        }
        
        Ok(())
    }
    
    /// Compare two hardware configurations to check if they are equal
    fn hardware_equals(&self, hw1: &Hardware, hw2: &Hardware) -> bool {
        hw1.semantically_eq(hw2)
    }
    
    /// Get hardware information for a client
    pub async fn get_hardware(&self, client_id: &str) -> CmdbResult<Option<Hardware>> {
        let hardware_data = match self.db.get(&self.get_key(client_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let hardware = serde_json::from_slice(&hardware_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize hardware: {}", e)))?;
            
        Ok(Some(hardware))
    }
    
    /// Get hardware history for a client
    pub async fn get_hardware_history(&self, client_id: &str) -> CmdbResult<Vec<(String, Hardware)>> {
        let history_prefix = format!("{}{}:history:", self.key_prefix, client_id);
        let entries = self.db.list_entries(&history_prefix).await?;
        let mut history = Vec::with_capacity(entries.len());
        
        for (key, data) in entries {
            let hardware = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize hardware history: {}", e)))?;
            
            // Extract timestamp from key
            let timestamp = key.strip_prefix(&history_prefix)
                .unwrap_or_default()
                .to_string();
            
            history.push((timestamp, hardware));
        }
        
        // Sort by timestamp, descending
        history.sort_by(|a, b| b.0.cmp(&a.0));
        
        Ok(history)
    }
    
    /// Delete hardware information for a client
    pub async fn delete_hardware(&self, client_id: &str) -> CmdbResult<()> {
        // Delete current hardware data
        self.db.delete(&self.get_key(client_id)).await?;
        
        // Delete history
        let history_prefix = format!("{}{}:history:", self.key_prefix, client_id);
        let keys = self.db.list_keys(&history_prefix).await?;
        
        for key in keys {
            self.db.delete(&key).await?;
        }
        
        Ok(())
    }
} 