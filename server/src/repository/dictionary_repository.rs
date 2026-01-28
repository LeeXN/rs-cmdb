use crate::db::Database;
use common::entity::dictionary::Dictionary;
use common::error::{CmdbError, CmdbResult};
use serde_json;
use std::sync::Arc;

/// Repository for dictionary operations
pub struct DictionaryRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl DictionaryRepository {
    /// Create a new dictionary repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "dictionary:".to_string(),
        }
    }

    /// Get dictionary key in database
    fn get_key(&self, id: &str) -> String {
        format!("{}{}", self.key_prefix, id)
    }

    /// Save a dictionary item to the database
    pub async fn save(&self, item: &Dictionary) -> CmdbResult<()> {
        let json = serde_json::to_vec(item).map_err(|e| {
            CmdbError::Serialization(format!("Failed to serialize dictionary item: {}", e))
        })?;

        self.db.set(&self.get_key(&item.id), &json).await
    }

    /// Get a dictionary item by ID
    pub async fn get(&self, id: &str) -> CmdbResult<Option<Dictionary>> {
        let data = match self.db.get(&self.get_key(id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };

        let item = serde_json::from_slice(&data).map_err(|e| {
            CmdbError::Serialization(format!("Failed to deserialize dictionary item: {}", e))
        })?;

        Ok(Some(item))
    }

    /// Delete a dictionary item
    pub async fn delete(&self, id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(id)).await
    }

    /// List all dictionary items
    pub async fn list_all(&self) -> CmdbResult<Vec<Dictionary>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut items = Vec::with_capacity(values.len());

        for data in values {
            let item = serde_json::from_slice(&data).map_err(|e| {
                CmdbError::Serialization(format!("Failed to deserialize dictionary item: {}", e))
            })?;
            items.push(item);
        }

        Ok(items)
    }

    /// List items by category
    pub async fn list_by_category(&self, category: &str) -> CmdbResult<Vec<Dictionary>> {
        let all = self.list_all().await?;
        Ok(all
            .into_iter()
            .filter(|item| item.category == category)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;

    fn create_test_dictionary(id: &str, category: &str, key: &str, value: &str) -> Dictionary {
        Dictionary {
            id: id.to_string(),
            category: category.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            description: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_create_dictionary_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        let dict = create_test_dictionary("dict-001", "Hardware", "CPU", "Intel Xeon");

        let result = repo.save(&dict).await;

        assert!(
            result.is_ok(),
            "Save should succeed with valid dictionary data"
        );
    }

    #[tokio::test]
    async fn test_get_dictionary_when_exists_then_returns_value() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        let dict = create_test_dictionary("dict-002", "OS", "kernel", "5.15.0");
        repo.save(&dict).await.unwrap();

        let result = repo.get("dict-002").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(dictionary)");
        let retrieved_dict = retrieved.unwrap();
        assert_eq!(retrieved_dict.category, "OS");
        assert_eq!(retrieved_dict.key, "kernel");
        assert_eq!(retrieved_dict.value, "5.15.0");
    }

    #[tokio::test]
    async fn test_get_dictionary_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        let result = repo.get("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(result.unwrap().is_none(), "Should return None");
    }

    #[tokio::test]
    async fn test_delete_dictionary_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        let dict = create_test_dictionary("dict-003", "Department", "HR", "Human Resources");
        repo.save(&dict).await.unwrap();

        let delete_result = repo.delete("dict-003").await;

        assert!(delete_result.is_ok(), "Delete should succeed");

        let get_result = repo.get("dict-003").await;
        assert!(
            get_result.unwrap().is_none(),
            "Dictionary should be deleted"
        );
    }

    #[tokio::test]
    async fn test_delete_dictionary_when_not_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        let result = repo.delete("nonexistent").await;

        assert!(result.is_ok(), "Delete should be idempotent");
    }

    #[tokio::test]
    async fn test_list_all_when_multiple_entries_then_returns_all() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        repo.save(&create_test_dictionary(
            "dict-004",
            "Hardware",
            "CPU",
            "Intel Xeon",
        ))
        .await
        .unwrap();
        repo.save(&create_test_dictionary(
            "dict-005", "OS", "kernel", "5.15.0",
        ))
        .await
        .unwrap();
        repo.save(&create_test_dictionary(
            "dict-006",
            "Department",
            "HR",
            "IT",
        ))
        .await
        .unwrap();

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        let items = result.unwrap();
        assert_eq!(items.len(), 3, "Should return all 3 entries");
    }

    #[tokio::test]
    async fn test_list_all_when_empty_then_returns_empty_vec() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        assert!(result.unwrap().is_empty(), "Should return empty vector");
    }

    #[tokio::test]
    async fn test_list_by_category_when_entries_match_then_returns_filtered() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        repo.save(&create_test_dictionary(
            "dict-007", "Hardware", "CPU", "Intel",
        ))
        .await
        .unwrap();
        repo.save(&create_test_dictionary(
            "dict-008", "Hardware", "Disk", "SSD",
        ))
        .await
        .unwrap();
        repo.save(&create_test_dictionary(
            "dict-009", "OS", "kernel", "5.15.0",
        ))
        .await
        .unwrap();

        let result = repo.list_by_category("Hardware").await;

        assert!(result.is_ok(), "List by category should succeed");
        let items = result.unwrap();
        assert_eq!(items.len(), 2, "Should return 2 Hardware entries");
    }

    #[tokio::test]
    async fn test_list_by_category_when_no_matches_then_returns_empty() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        repo.save(&create_test_dictionary(
            "dict-010", "OS", "kernel", "5.15.0",
        ))
        .await
        .unwrap();

        let result = repo.list_by_category("Department").await;

        assert!(result.is_ok(), "List by category should succeed");
        assert!(
            result.unwrap().is_empty(),
            "Should return empty for non-existent category"
        );
    }

    #[tokio::test]
    async fn test_update_dictionary_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = DictionaryRepository::new(std::sync::Arc::new(db));

        let dict = create_test_dictionary("dict-011", "Department", "HR", "Human Resources");
        repo.save(&dict).await.unwrap();

        let mut updated_dict = dict.clone();
        updated_dict.value = "Engineering".to_string();
        updated_dict.updated_at = chrono::Utc::now().to_rfc3339();

        let result = repo.save(&updated_dict).await;

        assert!(result.is_ok(), "Update should succeed");

        let retrieved = repo.get("dict-011").await.unwrap().unwrap();
        assert_eq!(retrieved.value, "Engineering", "Value should be updated");
        assert_ne!(
            retrieved.updated_at, dict.created_at,
            "Updated timestamp should differ"
        );
    }
}
