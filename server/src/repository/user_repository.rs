use crate::db::Database;
use common::entity::user::User;
use common::error::{CmdbError, CmdbResult};
use serde_json;
use std::sync::Arc;

/// Repository for user operations
pub struct UserRepository {
    db: Arc<dyn Database>,
    key_prefix: String,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            key_prefix: "user:".to_string(),
        }
    }

    /// Get user key in database
    fn get_key(&self, user_id: &str) -> String {
        format!("{}{}", self.key_prefix, user_id)
    }

    /// Save a user to the database
    pub async fn save(&self, user: &User) -> CmdbResult<()> {
        let user_json = serde_json::to_vec(user)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize user: {}", e)))?;

        self.db.set(&self.get_key(&user.id), &user_json).await
    }

    /// Get a user by ID
    pub async fn get(&self, user_id: &str) -> CmdbResult<Option<User>> {
        let user_data = match self.db.get(&self.get_key(user_id)).await? {
            Some(data) => data,
            None => return Ok(None),
        };

        let user = serde_json::from_slice(&user_data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize user: {}", e)))?;

        Ok(Some(user))
    }

    /// Find a user by username
    pub async fn find_by_username(&self, username: &str) -> CmdbResult<Option<User>> {
        // Since we don't have a secondary index, we iterate through all users
        // This is acceptable for a small number of users
        let users = self.list_all().await?;

        for user in users {
            if user.username == username {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    /// Check if a user exists
    #[allow(dead_code)]
    pub async fn exists(&self, user_id: &str) -> CmdbResult<bool> {
        self.db.exists(&self.get_key(user_id)).await
    }

    /// Delete a user
    pub async fn delete(&self, user_id: &str) -> CmdbResult<()> {
        self.db.delete(&self.get_key(user_id)).await
    }

    /// List all users
    pub async fn list_all(&self) -> CmdbResult<Vec<User>> {
        let values = self.db.list_values(&self.key_prefix).await?;
        let mut users = Vec::with_capacity(values.len());

        for data in values {
            let user = serde_json::from_slice(&data).map_err(|e| {
                CmdbError::Serialization(format!("Failed to deserialize user: {}", e))
            })?;
            users.push(user);
        }

        Ok(users)
    }

    /// Update a user
    pub async fn update(&self, user: &User) -> CmdbResult<()> {
        let user_json = serde_json::to_vec(user)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize user: {}", e)))?;
        self.db.set(&self.get_key(&user.id), &user_json).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::auth_service::AuthService;
    use crate::tests::fixtures::setup_test_db;
    use common::entity::user::Role;

    /// Helper function to create a test user
    fn create_test_user(id: &str, username: &str, role: Role) -> User {
        let auth_service = AuthService::new("test_secret".to_string());
        let password_hash = auth_service.hash_password("password123").unwrap();

        User {
            id: id.to_string(),
            username: username.to_string(),
            password_hash,
            role,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        }
    }

    #[tokio::test]
    async fn test_save_user_when_valid_data_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let user = create_test_user("user-001", "testuser1", Role::User);
        let result = repo.save(&user).await;

        assert!(result.is_ok(), "Save should succeed with valid user data");
    }

    #[tokio::test]
    async fn test_save_user_when_duplicate_id_then_replaces() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let user1 = create_test_user("user-001", "testuser1", Role::User);
        let mut user2 = create_test_user("user-001", "testuser2", Role::Admin);
        user2.password_hash = user1.password_hash.clone(); // Use same hash for comparison

        repo.save(&user1).await.unwrap();
        let result = repo.save(&user2).await;

        assert!(
            result.is_ok(),
            "Save with same ID should replace existing user"
        );

        let retrieved = repo.get("user-001").await.unwrap().unwrap();
        assert_eq!(
            retrieved.username, "testuser2",
            "Username should be updated"
        );
        assert_eq!(retrieved.role, Role::Admin, "Role should be updated");
    }

    #[tokio::test]
    async fn test_get_user_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let user = create_test_user("user-002", "testuser2", Role::User);
        repo.save(&user).await.unwrap();

        let result = repo.get("user-002").await;

        assert!(result.is_ok(), "Get should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(user)");
        let retrieved_user = retrieved.unwrap();
        assert_eq!(retrieved_user.id, "user-002");
        assert_eq!(retrieved_user.username, "testuser2");
    }

    #[tokio::test]
    async fn test_get_user_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let result = repo.get("nonexistent").await;

        assert!(result.is_ok(), "Get should not return error");
        assert!(result.unwrap().is_none(), "Should return None");
    }

    #[tokio::test]
    async fn test_find_by_username_when_exists_then_returns_some() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let user = create_test_user("user-003", "testuser3", Role::Admin);
        repo.save(&user).await.unwrap();

        let result = repo.find_by_username("testuser3").await;

        assert!(result.is_ok(), "Find by username should not return error");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Should return Some(user)");
        let retrieved_user = retrieved.unwrap();
        assert_eq!(retrieved_user.username, "testuser3");
        assert_eq!(retrieved_user.id, "user-003");
        assert_ne!(
            retrieved_user.password_hash, "password123",
            "Password should be hashed, not plaintext"
        );
    }

    #[tokio::test]
    async fn test_find_by_username_when_not_exists_then_returns_none() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let result = repo.find_by_username("nonexistent").await;

        assert!(result.is_ok(), "Find by username should not return error");
        assert!(result.unwrap().is_none(), "Should return None");
    }

    #[tokio::test]
    async fn test_list_all_when_multiple_users_then_returns_all() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        for i in 1..=5 {
            let user = create_test_user(
                &format!("user-00{}", i),
                &format!("testuser{}", i),
                Role::User,
            );
            repo.save(&user).await.unwrap();
        }

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        let users = result.unwrap();
        assert_eq!(users.len(), 5, "Should return all 5 users");
    }

    #[tokio::test]
    async fn test_list_all_when_empty_then_returns_empty_vec() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let result = repo.list_all().await;

        assert!(result.is_ok(), "List all should not return error");
        assert!(result.unwrap().is_empty(), "Should return empty vector");
    }

    #[tokio::test]
    async fn test_update_user_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let mut user = create_test_user("user-004", "testuser4", Role::User);
        repo.save(&user).await.unwrap();

        user.username = "updated_user".to_string();
        user.role = Role::Admin;
        user.is_active = false;

        let result = repo.update(&user).await;

        assert!(result.is_ok(), "Update should succeed");

        let retrieved = repo.get("user-004").await.unwrap().unwrap();
        assert_eq!(
            retrieved.username, "updated_user",
            "Username should be updated"
        );
        assert_eq!(
            retrieved.role,
            Role::Admin,
            "Role should be changed to Admin"
        );
        assert!(
            !retrieved.is_active,
            "Active status should be changed to false"
        );
    }

    #[tokio::test]
    async fn test_update_user_when_not_exists_then_still_saves() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let user = create_test_user("user-005", "newuser", Role::User);

        let result = repo.update(&user).await;

        assert!(
            result.is_ok(),
            "Update of non-existent user should succeed (create-or-update)"
        );

        let retrieved = repo.get("user-005").await.unwrap();
        assert!(retrieved.is_some(), "User should be created");
    }

    #[tokio::test]
    async fn test_delete_user_when_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let user = create_test_user("user-006", "testuser6", Role::User);
        repo.save(&user).await.unwrap();

        let delete_result = repo.delete("user-006").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        let get_result = repo.get("user-006").await;
        assert!(get_result.unwrap().is_none(), "User should be deleted");
    }

    #[tokio::test]
    async fn test_delete_user_when_not_exists_then_succeeds() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        // Deleting a non-existent user should not error (idempotent operation)
        let result = repo.delete("nonexistent").await;

        assert!(result.is_ok(), "Delete should be idempotent");
    }

    #[tokio::test]
    async fn test_exists_when_user_exists_then_returns_true() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let user = create_test_user("user-007", "testuser7", Role::User);
        repo.save(&user).await.unwrap();

        let result = repo.exists("user-007").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(result.unwrap(), "Should return true for existing user");
    }

    #[tokio::test]
    async fn test_exists_when_user_not_exists_then_returns_false() {
        let db = setup_test_db().unwrap();
        let repo = UserRepository::new(std::sync::Arc::new(db));

        let result = repo.exists("nonexistent").await;

        assert!(result.is_ok(), "Exists should not return error");
        assert!(
            !result.unwrap(),
            "Should return false for non-existent user"
        );
    }
}
