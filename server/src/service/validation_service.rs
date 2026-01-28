use std::sync::Arc;
use crate::repository::{
    client_repository::ClientRepository,
    project_repository::ProjectRepository,
    rack_repository::RackRepository,
    person_repository::PersonRepository
};
use common::error::{CmdbResult, CmdbError};

#[cfg(test)]
use crate::tests::fixtures::*;

#[derive(Clone)]
pub struct ValidationService {
    client_repo: Arc<ClientRepository>,
    project_repo: Arc<ProjectRepository>,
    rack_repo: Arc<RackRepository>,
    person_repo: Arc<PersonRepository>,
}

impl ValidationService {
    pub fn new(
        client_repo: Arc<ClientRepository>,
        project_repo: Arc<ProjectRepository>,
        rack_repo: Arc<RackRepository>,
        person_repo: Arc<PersonRepository>,
    ) -> Self {
        Self {
            client_repo,
            project_repo,
            rack_repo,
            person_repo,
        }
    }

    pub async fn validate_rack_exists(&self, rack_id: &str) -> CmdbResult<()> {
        if !self.rack_repo.exists(rack_id).await? {
            return Err(CmdbError::Validation(format!("Rack with ID {} not found", rack_id)));
        }
        Ok(())
    }

    pub async fn validate_project_exists(&self, project_id: &str) -> CmdbResult<()> {
        if !self.project_repo.exists(project_id).await? {
            return Err(CmdbError::Validation(format!("Project with ID {} not found", project_id)));
        }
        Ok(())
    }

    pub async fn validate_person_exists(&self, person_id: &str) -> CmdbResult<()> {
        if !self.person_repo.exists(person_id).await? {
            return Err(CmdbError::Validation(format!("Person with ID {} not found", person_id)));
        }
        Ok(())
    }
    
    pub async fn validate_client_exists(&self, client_id: &str) -> CmdbResult<()> {
        if self.client_repo.get(client_id).await?.is_none() {
             return Err(CmdbError::Validation(format!("Client with ID {} not found", client_id)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::repository::{
        client_repository::ClientRepository,
        hardware_repository::HardwareRepository,
        project_repository::ProjectRepository,
        rack_repository::RackRepository,
        person_repository::PersonRepository
    };
    use crate::tests::fixtures::*;

    #[tokio::test]
    async fn test_validation_service_creation() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let _service = ValidationService::new(client_repo, project_repo, rack_repo, person_repo);
    }

    #[tokio::test]
    async fn test_validate_rack_exists_with_valid_rack() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let rack = create_test_rack("rack-1");
        rack_repo.save(&rack).await.unwrap();

        let result = service.validate_rack_exists("rack-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_rack_exists_with_invalid_rack() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let result = service.validate_rack_exists("nonexistent-rack").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_project_exists_with_valid_project() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let project = create_test_project("project-1");
        project_repo.save(&project).await.unwrap();

        let result = service.validate_project_exists("project-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_project_exists_with_invalid_project() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let result = service.validate_project_exists("nonexistent-project").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_person_exists_with_valid_person() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let person = create_test_person("person-1");
        person_repo.save(&person).await.unwrap();

        let result = service.validate_person_exists("person-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_person_exists_with_invalid_person() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let result = service.validate_person_exists("nonexistent-person").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_client_exists_with_valid_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let client = create_test_client("client-1");
        client_repo.save(&client).await.unwrap();

        let result = service.validate_client_exists("client-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_client_exists_with_invalid_client() {
        let db = setup_test_db().unwrap();
        let db_arc: Arc<dyn crate::db::Database> = Arc::new(db);
        let client_repo = Arc::new(ClientRepository::new(Arc::clone(&db_arc)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&db_arc)));
        let rack_repo = Arc::new(RackRepository::new(Arc::clone(&db_arc)));
        let person_repo = Arc::new(PersonRepository::new(Arc::clone(&db_arc)));

        let service = ValidationService::new(client_repo.clone(), project_repo.clone(), rack_repo.clone(), person_repo.clone());

        let result = service.validate_client_exists("nonexistent-client").await;

        assert!(result.is_err());
    }
}
