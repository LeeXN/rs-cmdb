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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::models::{Component, ComponentQuery, ComponentStatus, ComponentType};

    fn create_test_component(id: &str, component_type: ComponentType) -> Component {
        Component {
            id: id.to_string(),
            serial_number: format!("SN-{}", id),
            model: format!("Model-{}", id),
            vendor: Some(format!("Vendor-{}", id)),
            component_type,
            status: ComponentStatus::InStock,
            client_id: Some("client-001".to_string()),
            client_hostname: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            location: None,
            purchase_date: None,
            warranty_expiration: None,
            missing_since: None,
        }
    }

    #[tokio::test]
    async fn test_create_component_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let component = create_test_component("comp-001", ComponentType::CPU);
        let result = repo.save(&component).await;

        assert!(result.is_ok(), "Save should succeed with valid component data");
    }

    #[tokio::test]
    async fn test_get_component_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let component = create_test_component("comp-002", ComponentType::Memory);
        repo.save(&component).await.unwrap();

        let result = repo.get("comp-002").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(component)");
        assert_eq!(retrieved.unwrap().id, "comp-002");
    }

    #[tokio::test]
    async fn test_get_component_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let result = repo.get("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(result.unwrap().is_none(), "Should return None");
    }

    #[tokio::test]
    async fn test_find_with_query_filters_by_component_type() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let cpu = create_test_component("cpu-001", ComponentType::CPU);
        let memory = create_test_component("mem-001", ComponentType::Memory);
        let gpu = create_test_component("gpu-001", ComponentType::GPU);

        repo.save(&cpu).await.unwrap();
        repo.save(&memory).await.unwrap();
        repo.save(&gpu).await.unwrap();

        let query = ComponentQuery {
            component_type: Some(ComponentType::CPU),
            page: Some(1),
            page_size: Some(1),
            ..Default::default()
        };

        let result = repo.find_with_query(query).await;

        assert!(result.is_ok(), "Find with query should succeed");
        let paginated = result.unwrap();
        assert_eq!(paginated.items.len(), 1, "Should return only CPU components");
        assert_eq!(paginated.total, 1, "Total should be 1 (only CPU component matches)");
        assert_eq!(paginated.page, 1, "Should be page 1");
        assert_eq!(paginated.page_size, 1, "Page size should be 1");
        assert_eq!(paginated.total_pages, 1, "Should have 1 page");
    }

    #[tokio::test]
    async fn test_find_with_query_filters_by_client_id() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let comp1 = create_test_component("comp-003", ComponentType::CPU);
        let mut comp2 = create_test_component("comp-004", ComponentType::CPU);
        comp2.client_id = Some("client-002".to_string());

        repo.save(&comp1).await.unwrap();
        repo.save(&comp2).await.unwrap();

        let query = ComponentQuery {
            client_id: Some("client-001".to_string()),
            ..Default::default()
        };

        let result = repo.find_with_query(query).await;

        assert!(result.is_ok(), "Find with query should succeed");
        let paginated = result.unwrap();
        assert_eq!(paginated.items.len(), 1, "Should return only components for client-001");
        assert_eq!(paginated.items[0].id, "comp-003");
    }

    #[tokio::test]
    async fn test_find_with_query_pagination_returns_correct_page() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        for i in 1..=15 {
            let component = create_test_component(&format!("comp-{:03}", i), ComponentType::Memory);
            repo.save(&component).await.unwrap();
        }

        let query = ComponentQuery {
            page: Some(2),
            page_size: Some(5),
            ..Default::default()
        };

        let result = repo.find_with_query(query).await;

        assert!(result.is_ok(), "Find with query should succeed");
        let paginated = result.unwrap();
        assert_eq!(paginated.items.len(), 5, "Should return 5 items on page 2");
        assert_eq!(paginated.page, 2, "Should be page 2");
        assert_eq!(paginated.total, 15, "Total should be 15");
        assert_eq!(paginated.total_pages, 3, "Should have 3 pages");
    }

    #[tokio::test]
    async fn test_find_with_query_pagination_out_of_bounds_returns_empty() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        for i in 1..=5 {
            let component = create_test_component(&format!("comp-{:04}", i), ComponentType::Disk);
            repo.save(&component).await.unwrap();
        }

        let query = ComponentQuery {
            page: Some(10),
            page_size: Some(5),
            ..Default::default()
        };

        let result = repo.find_with_query(query).await;

        assert!(result.is_ok(), "Find with query should succeed");
        let paginated = result.unwrap();
        assert_eq!(paginated.items.len(), 0, "Should return empty for out of bounds page");
        assert_eq!(paginated.total, 5, "Total should still be 5");
    }

    #[tokio::test]
    async fn test_find_with_query_combines_multiple_filters() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let cpu = create_test_component("comp-005", ComponentType::CPU);
        let mut memory = create_test_component("comp-006", ComponentType::Memory);
        memory.status = ComponentStatus::InUse;

        repo.save(&cpu).await.unwrap();
        repo.save(&memory).await.unwrap();

        let query = ComponentQuery {
            component_type: Some(ComponentType::Memory),
            status: Some(ComponentStatus::InUse),
            ..Default::default()
        };

        let result = repo.find_with_query(query).await;

        assert!(result.is_ok(), "Find with query should succeed");
        let paginated = result.unwrap();
        assert_eq!(paginated.items.len(), 1, "Should return only InUse memory");
        assert_eq!(paginated.items[0].id, "comp-006");
    }

    #[tokio::test]
    async fn test_update_component_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let mut component = create_test_component("comp-007", ComponentType::GPU);
        repo.save(&component).await.unwrap();

        component.model = "Updated Model".to_string();

        let result = repo.save(&component).await;

        assert!(result.is_ok(), "Update should succeed");

        let retrieved = repo.get("comp-007").await.unwrap().unwrap();
        assert_eq!(retrieved.model, "Updated Model");
        assert_ne!(retrieved.updated_at, retrieved.created_at, "Updated timestamp should differ");
    }

    #[tokio::test]
    async fn test_delete_component_when_exists_then_removes() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let component = create_test_component("comp-008", ComponentType::CPU);
        repo.save(&component).await.unwrap();

        let result = repo.delete("comp-008").await;

        assert!(result.is_ok(), "Delete should succeed");

        let get_result = repo.get("comp-008").await;
        assert!(get_result.unwrap().is_none(), "Component should be deleted");
    }

    #[tokio::test]
    async fn test_exists_when_component_exists_then_returns_true() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let component = create_test_component("comp-009", ComponentType::Disk);
        repo.save(&component).await.unwrap();

        let result = repo.exists("comp-009").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(result.unwrap(), "Should return true for existing component");
    }

    #[tokio::test]
    async fn test_exists_when_component_not_exists_then_returns_false() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let result = repo.exists("nonexistent").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(!result.unwrap(), "Should return false for non-existent component");
    }

    #[tokio::test]
    async fn test_release_components_by_client_sets_status_to_instock() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let mut component1 = create_test_component("comp-010", ComponentType::CPU);
        component1.client_id = Some("client-001".to_string());

        let mut component2 = create_test_component("comp-011", ComponentType::Memory);
        component2.client_id = Some("client-001".to_string());

        repo.save(&component1).await.unwrap();
        repo.save(&component2).await.unwrap();

        let result = repo.release_components_by_client("client-001").await;

        assert!(result.is_ok(), "Release should succeed");

        let comp1 = repo.get("comp-010").await.unwrap().unwrap();
        let comp2 = repo.get("comp-011").await.unwrap().unwrap();

        assert_eq!(comp1.client_id, None, "Client ID should be set to None");
        assert_eq!(comp2.client_id, None, "Client ID should be set to None");
        assert_eq!(comp1.status, ComponentStatus::InStock, "Status should be InStock");
        assert_eq!(comp2.status, ComponentStatus::InStock, "Status should be InStock");
    }

    #[tokio::test]
    async fn test_find_by_serial_when_exists_then_returns_component() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let mut component = create_test_component("comp-012", ComponentType::Disk);
        component.serial_number = "UNIQUE-SN-12345".to_string();
        repo.save(&component).await.unwrap();

        let result = repo.find_by_serial("UNIQUE-SN-12345").await;

        assert!(result.is_ok(), "Find by serial should succeed");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(component)");
        assert_eq!(retrieved.unwrap().id, "comp-012");
    }

    #[tokio::test]
    async fn test_find_by_serial_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = ComponentRepository::new(std::sync::Arc::new(db));

        let component = create_test_component("comp-013", ComponentType::CPU);
        repo.save(&component).await.unwrap();

        let result = repo.find_by_serial("NONEXISTENT").await;

        assert!(result.is_ok(), "Find by serial should succeed");
        assert!(result.unwrap().is_none(), "Should return None");
    }
}
