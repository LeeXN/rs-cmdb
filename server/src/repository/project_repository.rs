use std::sync::Arc;
use common::error::{CmdbResult, CmdbError};
use common::models::Project;
use serde_json;
use crate::db::Database;

/// Repository for project operations
pub struct ProjectRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl ProjectRepository {
    /// Create a new project repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "project:".to_string(),
        }
    }
    
    /// Get project key in database
    fn get_key(&self, project_id: &str) -> String {
        format!("{}{}", self.key_prefix, project_id)
    }
    
    /// Save a project to the database
    pub async fn save(&self, project: &Project) -> CmdbResult<()> {
        let project_json = serde_json::to_vec(project)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize project: {}", e)))?;
        
        self.db.set(&self.get_key(&project.id), &project_json).await
    }
    
    /// Get a project by ID
    pub async fn get(&self, project_id: &str) -> CmdbResult<Option<Project>> {
        let project_data = match self.db.get(&self.get_key(project_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let project = serde_json::from_slice(&project_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize project: {}", e)))?;
            
        Ok(Some(project))
    }
    
    /// Check if a project exists
    pub async fn exists(&self, project_id: &str) -> CmdbResult<bool> {
        self.db.exists(&self.get_key(project_id)).await
    }
    
    /// Delete a project
    pub async fn delete(&self, project_id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(project_id)).await
    }
    
    /// List all projects
    pub async fn list_all(&self) -> CmdbResult<Vec<Project>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut projects = Vec::with_capacity(values.len());
        
        for data in values {
            let project = serde_json::from_slice(&data)
                .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize project: {}", e)))?;
            projects.push(project);
        }
        
        Ok(projects)
    }

    /// Update manager to null for projects managed by a specific person
    pub async fn update_manager_to_null(&self, manager_id: &str) -> CmdbResult<()> {
        let projects = self.list_all().await?;
        for mut project in projects {
            if project.manager_id.as_deref() == Some(manager_id) {
                project.manager_id = None;
                self.save(&project).await?;
            }
        }
        Ok(())
    }
}
