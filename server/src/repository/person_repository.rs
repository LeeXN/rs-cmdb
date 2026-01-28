use crate::db::Database;
use common::error::{CmdbError, CmdbResult};
use common::models::Person;
use serde_json;
use std::sync::Arc;

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

        let person = serde_json::from_slice(&person_data).map_err(|e| {
            CmdbError::Serialization(format!("Failed to deserialize person: {}", e))
        })?;

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
            let person = serde_json::from_slice(&data).map_err(|e| {
                CmdbError::Serialization(format!("Failed to deserialize person: {}", e))
            })?;
            persons.push(person);
        }

        Ok(persons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::models::Person;

    fn create_test_person(id: &str, name: &str) -> Person {
        Person {
            id: id.to_string(),
            name: name.to_string(),
            email: format!("{}@example.com", name),
            phone: Some(format!("555-010{}", id)),
            department: Some("Engineering".to_string()),
            title: Some("Developer".to_string()),
            cost_center: Some("CC-001".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_save_person_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let person = create_test_person("person-001", "John Doe");
        let result = repo.save(&person).await;

        assert!(result.is_ok(), "Save should succeed");
    }

    #[tokio::test]
    async fn test_get_person_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let person = create_test_person("person-002", "Jane Smith");
        repo.save(&person).await.unwrap();

        let result = repo.get("person-002").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(person)");
        assert_eq!(retrieved.unwrap().name, "Jane Smith");
    }

    #[tokio::test]
    async fn test_get_person_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let result = repo.get("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(
            result.unwrap().is_none(),
            "Should return None for non-existent person"
        );
    }

    #[tokio::test]
    async fn test_exists_when_person_exists_then_returns_true() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let person = create_test_person("person-003", "Bob Johnson");
        repo.save(&person).await.unwrap();

        let result = repo.exists("person-003").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(result.unwrap(), "Should return true for existing person");
    }

    #[tokio::test]
    async fn test_exists_when_person_not_exists_then_returns_false() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let result = repo.exists("nonexistent").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(
            !result.unwrap(),
            "Should return false for non-existent person"
        );
    }

    #[tokio::test]
    async fn test_delete_person_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let person = create_test_person("person-004", "Alice Brown");
        repo.save(&person).await.unwrap();

        let delete_result = repo.delete("person-004").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        let get_result = repo.get("person-004").await;
        assert!(get_result.unwrap().is_none(), "Person should be deleted");
    }

    #[tokio::test]
    async fn test_delete_person_when_not_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let result = repo.delete("nonexistent").await;

        assert!(result.is_ok(), "Delete should be idempotent");
    }

    #[tokio::test]
    async fn test_list_all_when_multiple_persons_then_returns_all() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let person1 = create_test_person("person-005", "Charlie Davis");
        let person2 = create_test_person("person-006", "Diana Evans");
        let person3 = create_test_person("person-007", "Frank Wright");

        repo.save(&person1).await.unwrap();
        repo.save(&person2).await.unwrap();
        repo.save(&person3).await.unwrap();

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        let persons = result.unwrap();
        assert_eq!(persons.len(), 3, "Should return all persons");
    }

    #[tokio::test]
    async fn test_list_all_when_empty_then_returns_empty_vec() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        assert!(
            result.unwrap().is_empty(),
            "Should return empty vec for empty db"
        );
    }

    #[tokio::test]
    async fn test_save_person_with_duplicate_id_then_replaces() {
        let db = setup_test_db().unwrap();
        let repo = PersonRepository::new(std::sync::Arc::new(db));

        let person1 = create_test_person("person-008", "Grace Hill");
        let mut person2 = create_test_person("person-008", "Grace Updated");
        person2.email = "grace.updated@example.com".to_string();

        repo.save(&person1).await.unwrap();
        repo.save(&person2).await.unwrap();

        let result = repo.get("person-008").await;
        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(person)");
        assert_eq!(
            retrieved.unwrap().email,
            "grace.updated@example.com",
            "Should have updated email"
        );
    }
}
