use crate::db::Database;
use common::error::{CmdbError, CmdbResult};
use common::models::Project;
use serde_json;
use std::sync::Arc;

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

        let project = serde_json::from_slice(&project_data).map_err(|e| {
            CmdbError::Serialization(format!("Failed to deserialize project: {}", e))
        })?;

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
            let project = serde_json::from_slice(&data).map_err(|e| {
                CmdbError::Serialization(format!("Failed to deserialize project: {}", e))
            })?;
            projects.push(project);
        }

        Ok(projects)
    }

    /// Update manager to null for projects managed by a specific person
    pub async fn update_manager_to_null(&self, manager_id: &str) -> CmdbResult<()> {
        let manager_id = manager_id.to_string();
        
        self.db.update_all(
            &self.key_prefix,
            Box::new(move |_key, value| {
                match serde_json::from_slice::<Project>(&value) {
                    Ok(mut project) => {
                        if project.manager_id.as_deref() == Some(&manager_id) {
                            project.manager_id = None;
                            match serde_json::to_vec(&project) {
                                Ok(new_value) => Some(new_value),
                                Err(_) => None // Skip update if serialization fails
                            }
                        } else {
                            None
                        }
                    }
                    Err(_) => None // Skip update if deserialization fails
                }
            })
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::models::Project;

    fn create_test_project(id: &str, name: &str) -> Project {
        Project {
            id: id.to_string(),
            name: name.to_string(),
            code: Some(format!("PROJ-{}", id)),
            department: Some("Engineering".to_string()),
            cost_center: Some("CC-001".to_string()),
            manager_id: Some("manager-001".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_save_project_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let project = create_test_project("project-001", "Web Platform");
        let result = repo.save(&project).await;

        assert!(result.is_ok(), "Save should succeed");
    }

    #[tokio::test]
    async fn test_get_project_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let project = create_test_project("project-002", "Mobile App");
        repo.save(&project).await.unwrap();

        let result = repo.get("project-002").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(project)");
        assert_eq!(retrieved.unwrap().name, "Mobile App");
    }

    #[tokio::test]
    async fn test_get_project_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let result = repo.get("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(
            result.unwrap().is_none(),
            "Should return None for non-existent project"
        );
    }

    #[tokio::test]
    async fn test_exists_when_project_exists_then_returns_true() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let project = create_test_project("project-003", "API Gateway");
        repo.save(&project).await.unwrap();

        let result = repo.exists("project-003").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(result.unwrap(), "Should return true for existing project");
    }

    #[tokio::test]
    async fn test_exists_when_project_not_exists_then_returns_false() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let result = repo.exists("nonexistent").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(
            !result.unwrap(),
            "Should return false for non-existent project"
        );
    }

    #[tokio::test]
    async fn test_delete_project_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let project = create_test_project("project-004", "Legacy System");
        repo.save(&project).await.unwrap();

        let delete_result = repo.delete("project-004").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        let get_result = repo.get("project-004").await;
        assert!(get_result.unwrap().is_none(), "Project should be deleted");
    }

    #[tokio::test]
    async fn test_delete_project_when_not_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let result = repo.delete("nonexistent").await;

        assert!(result.is_ok(), "Delete should be idempotent");
    }

    #[tokio::test]
    async fn test_list_all_when_multiple_projects_then_returns_all() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let project1 = create_test_project("project-005", "Database Migration");
        let project2 = create_test_project("project-006", "Security Audit");
        let project3 = create_test_project("project-007", "Infrastructure Upgrade");

        repo.save(&project1).await.unwrap();
        repo.save(&project2).await.unwrap();
        repo.save(&project3).await.unwrap();

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        let projects = result.unwrap();
        assert_eq!(projects.len(), 3, "Should return all projects");
    }

    #[tokio::test]
    async fn test_list_all_when_empty_then_returns_empty_vec() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        assert!(
            result.unwrap().is_empty(),
            "Should return empty vec for empty db"
        );
    }

    #[tokio::test]
    async fn test_save_project_with_duplicate_id_then_replaces() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let project1 = create_test_project("project-008", "Original Project");
        let mut project2 = create_test_project("project-008", "Updated Project");
        project2.code = Some("PROJ-UPDATED".to_string());

        repo.save(&project1).await.unwrap();
        repo.save(&project2).await.unwrap();

        let result = repo.get("project-008").await;
        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(project)");
        assert_eq!(
            retrieved.unwrap().code,
            Some("PROJ-UPDATED".to_string()),
            "Should have updated code"
        );
    }

    #[tokio::test]
    async fn test_update_manager_to_null_when_manager_has_projects() {
        let db = setup_test_db().unwrap();
        let repo = ProjectRepository::new(std::sync::Arc::new(db));

        let manager_id = "manager-999";
        let mut project1 = create_test_project("project-009", "Project A");
        let mut project2 = create_test_project("project-010", "Project B");
        let mut project3 = create_test_project("project-011", "Project C");

        project1.manager_id = Some(manager_id.to_string());
        project2.manager_id = Some(manager_id.to_string());
        project3.manager_id = Some("other-manager".to_string());

        repo.save(&project1).await.unwrap();
        repo.save(&project2).await.unwrap();
        repo.save(&project3).await.unwrap();

        let result = repo.update_manager_to_null(manager_id).await;

        assert!(result.is_ok(), "Update should succeed");

        let project1_updated = repo.get("project-009").await.unwrap().unwrap();
        let project2_updated = repo.get("project-010").await.unwrap().unwrap();
        let project3_updated = repo.get("project-011").await.unwrap().unwrap();

        assert!(
            project1_updated.manager_id.is_none(),
            "Project A manager should be null"
        );
        assert!(
            project2_updated.manager_id.is_none(),
            "Project B manager should be null"
        );
        assert_eq!(
            project3_updated.manager_id,
            Some("other-manager".to_string()),
            "Project C manager should not be null"
        );
    }
}
