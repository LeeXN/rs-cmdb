use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::models::{Component, ComponentQuery, PaginatedResult, ComponentStatus};
use serde_json;
use crate::db::Database;

/// Repository for component operations
pub struct ComponentRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl ComponentRepository {
    /// Create a new component repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "component:".to_string(),
        }
    }
    
    /// Get component key in database
    fn get_key(&self, component_id: &str) -> String {
        format!("{}{}", self.key_prefix, component_id)
    }
    
    /// Save a component to the database
    pub async fn save(&self, component: &Component) -> CmdbResult<()> {
        let component_json = serde_json::to_vec(component)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize component: {}", e)))?;
        
        self.db.set(&self.get_key(&component.id), &component_json).await
    }
    
    /// Get a component by ID
    pub async fn get(&self, component_id: &str) -> CmdbResult<Option<Component>> {
        let component_data = match self.db.get(&self.get_key(component_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let component = serde_json::from_slice(&component_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize component: {}", e)))?;
            
        Ok(Some(component))
    }
    
    /// Check if a component exists
    pub async fn exists(&self, component_id: &str) -> CmdbResult<bool> {
        self.db.exists(&self.get_key(component_id)).await
    }
    
    /// Delete a component
    pub async fn delete(&self, component_id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(component_id)).await
    }
    
    /// List all components
    pub async fn list_all(&self) -> CmdbResult<Vec<Component>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut components = Vec::with_capacity(values.len());
        
        for data in values {
            let component = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize component: {}", e)))?;
            components.push(component);
        }
        
        Ok(components)
    }

    /// Find component by serial number (This is inefficient in KV store without index, but okay for small scale)
    pub async fn find_by_serial(&self, serial: &str) -> CmdbResult<Option<Component>> {
        let all = self.list_all().await?;
        Ok(all.into_iter().find(|c| c.serial_number == serial))
    }

    /// Find components by client ID
    pub async fn find_by_client_id(&self, client_id: &str) -> CmdbResult<Vec<Component>> {
        let all = self.list_all().await?;
        Ok(all.into_iter().filter(|c| c.client_id.as_deref() == Some(client_id)).collect())
    }

    /// Release components by client ID (set client_id to None and status to InStock)
    pub async fn release_components_by_client(&self, client_id: &str) -> CmdbResult<()> {
        let components = self.find_by_client_id(client_id).await?;
        for mut component in components {
            component.client_id = None;
            component.status = ComponentStatus::InStock;
            self.save(&component).await?;
        }
        Ok(())
    }

    /// Find components with query filters and pagination
    pub async fn find_with_query(&self, query: ComponentQuery) -> CmdbResult<PaginatedResult<Component>> {
        let mut components = self.list_all().await?;
        
        // Filter by client_id
        if let Some(client_id) = &query.client_id {
            components.retain(|c| c.client_id.as_deref() == Some(client_id));
        }

        // Filter by status
        if let Some(status) = &query.status {
            components.retain(|c| c.status == *status);
        }
        
        // Filter by type
        if let Some(component_type) = &query.component_type {
            components.retain(|c| c.component_type == *component_type);
        }
        
        // Filter by search
        if let Some(search) = &query.search {
            if !search.is_empty() {
                let search_lower = search.to_lowercase();
                components.retain(|c| {
                    c.serial_number.to_lowercase().contains(&search_lower) ||
                    c.model.to_lowercase().contains(&search_lower) ||
                    c.vendor.as_ref().map(|v| v.to_lowercase().contains(&search_lower)).unwrap_or(false)
                });
            }
        }
        
        let total = components.len();
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);
        let total_pages = (total as f64 / page_size as f64).ceil() as usize;
        
        let start = (page - 1) * page_size;
        let end = std::cmp::min(start + page_size, total);
        
        let items = if start < total {
            components[start..end].to_vec()
        } else {
            Vec::new()
        };
        
        Ok(PaginatedResult {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
