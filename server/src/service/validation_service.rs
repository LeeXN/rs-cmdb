use std::sync::Arc;
use crate::repository::{
    client_repository::ClientRepository,
    project_repository::ProjectRepository,
    rack_repository::RackRepository,
    person_repository::PersonRepository,
};
use common::error::{CmdbResult, CmdbError};

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
