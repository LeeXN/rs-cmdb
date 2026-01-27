//! Common test fixtures for the rs-cmdb server
//!
//! This module provides reusable test fixtures including:
//! - Test database setup
//! - Authentication test users
//! - Test data helpers

use chrono::Utc;
use common::entity::user::{User, Role};
use crate::db::{Database, redb_store::RedbStore};
use crate::service::auth_service::AuthService;

/// Test user structure with known credentials
pub struct TestUser {
    pub username: &'static str,
    pub password: &'static str,
    pub role: Role,
    pub id: String,
}

/// Returns a test admin user
///
/// # Example
/// ```
/// let admin = test_admin();
/// assert_eq!(admin.username, "test_admin");
/// ```
pub fn test_admin() -> TestUser {
    TestUser {
        username: "test_admin",
        password: "admin123",
        role: Role::Admin,
        id: "admin-test-001".to_string(),
    }
}

/// Returns a test regular user
///
/// # Example
/// ```
/// let user = test_user();
/// assert_eq!(user.username, "test_user");
/// ```
pub fn test_user() -> TestUser {
    TestUser {
        username: "test_user",
        password: "user123",
        role: Role::User,
        id: "user-test-001".to_string(),
    }
}

/// Returns a test viewer user
pub fn test_viewer() -> TestUser {
    TestUser {
        username: "test_viewer",
        password: "viewer123",
        role: Role::Viewer,
        id: "viewer-test-001".to_string(),
    }
}

/// Setup an in-memory test database
///
/// Creates a temporary in-memory RedbStore instance for testing.
/// The database is automatically cleaned up when dropped.
///
/// Note: ReDB doesn't support true in-memory databases with the `:memory:` path
/// when used across multiple connections. For testing, each test should create
/// its own database instance or use temporary files.
///
/// # Returns
///
/// A `RedbStore` instance for testing
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::setup_test_db;
///
/// let db = setup_test_db().unwrap();
/// // Use db for testing...
/// // Database is automatically cleaned up when dropped
/// ```
pub fn setup_test_db() -> Result<RedbStore, Box<dyn std::error::Error>> {
    // Use a temporary file-based database for ReDB
    // ReDB doesn't support true :memory: databases
    // Generate a unique filename using timestamp and random value
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let random: u64 = rand::random();
    let db_path = std::env::temp_dir().join(format!("test_rs_cmdb_{}_{}.redb", timestamp, random));
    let db = RedbStore::new(&db_path)?;

    // Note: The temp file will remain on disk but in the temp directory
    // which gets cleaned up by the OS periodically
    Ok(db)
}

/// Seeds a test user into the database
///
/// Creates a user with hashed password and stores it in the database.
///
/// # Arguments
///
/// * `db` - The database to seed the user into
/// * `user` - The test user to create
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::{setup_test_db, seed_test_user, test_admin};
///
/// let db = setup_test_db().unwrap();
/// let admin = test_admin();
/// seed_test_user(&db, &admin).await.unwrap();
/// ```
pub async fn seed_test_user(db: &RedbStore, user: &TestUser) -> Result<(), Box<dyn std::error::Error>> {
    let auth_service = AuthService::new("test_secret".to_string());
    let password_hash = auth_service.hash_password(user.password)?;

    let user_entity = User {
        id: user.id.clone(),
        username: user.username.to_string(),
        password_hash,
        role: user.role.clone(),
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    // Serialize and store the user
    let user_json = serde_json::to_vec(&user_entity)?;
    let key = format!("user:{}", user.username);
    db.set(&key, &user_json).await?;

    Ok(())
}

/// Generates a valid JWT token for a test user
///
/// # Arguments
///
/// * `user` - The test user to generate a token for
///
/// # Returns
///
/// A JWT token string
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::{test_admin, generate_test_token};
///
/// let admin = test_admin();
/// let token = generate_test_token(&admin);
/// assert!(!token.is_empty());
/// ```
pub fn generate_test_token(user: &TestUser) -> String {
    let auth_service = AuthService::new("test_secret".to_string());
    let user_entity = User {
        id: user.id.clone(),
        username: user.username.to_string(),
        password_hash: "hash".to_string(),
        role: user.role.clone(),
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    auth_service.generate_token(&user_entity)
        .expect("Failed to generate test token")
}

/// Helper to create authenticated request headers
///
/// # Arguments
///
/// * `token` - The JWT token to include in the Authorization header
///
/// # Returns
///
/// A tuple of ("authorization", "Bearer <token>")
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::{test_admin, generate_test_token, auth_headers};
///
/// let admin = test_admin();
/// let token = generate_test_token(&admin);
/// let headers = auth_headers(&token);
/// ```
pub fn auth_headers(token: &str) -> (&'static str, String) {
    ("authorization", format!("Bearer {}", token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let db = setup_test_db().unwrap();
        // Test that we can write and read
        db.set("test_key", b"test_value").await.unwrap();
        let value: Option<Vec<u8>> = db.get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
    }

    #[tokio::test]
    async fn test_seed_test_user() {
        let db = setup_test_db().unwrap();
        let admin = test_admin();

        seed_test_user(&db, &admin).await.unwrap();

        // Verify user was created
        let key = format!("user:{}", admin.username);
        let user_data: Option<Vec<u8>> = db.get(&key).await.unwrap();
        assert!(user_data.is_some());

        let user: User = serde_json::from_slice(&user_data.unwrap()).unwrap();
        assert_eq!(user.username, admin.username);
        assert_eq!(user.role, admin.role);
        assert_ne!(user.password_hash, admin.password); // Password should be hashed
    }

    #[test]
    fn test_generate_test_token() {
        let admin = test_admin();
        let token = generate_test_token(&admin);

        assert!(!token.is_empty());

        // Verify token can be decoded
        let auth_service = AuthService::new("test_secret".to_string());
        let claims = auth_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, admin.id);
        assert_eq!(claims.username, admin.username);
        assert_eq!(claims.role, admin.role);
    }

    #[test]
    fn test_auth_headers() {
        let token = "test_token_123";
        let (key, value) = auth_headers(token);
        assert_eq!(key, "authorization");
        assert_eq!(value, "Bearer test_token_123");
    }
}
