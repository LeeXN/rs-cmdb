use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dictionary {
    pub id: String,
    pub category: String, // "Department", "Title", "CostCenter"
    pub key: String,      // Unique key within category, e.g. "IT", "HR"
    pub value: String,    // Display value, e.g. "Information Technology"
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for Dictionary {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            category: String::new(),
            key: String::new(),
            value: String::new(),
            description: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_default() {
        let dict = Dictionary::default();
        assert!(!dict.id.is_empty());
        assert!(dict.category.is_empty());
        assert!(dict.key.is_empty());
        assert!(dict.value.is_empty());
        assert!(dict.description.is_none());
        assert!(!dict.created_at.is_empty());
        assert!(!dict.updated_at.is_empty());
    }
}
