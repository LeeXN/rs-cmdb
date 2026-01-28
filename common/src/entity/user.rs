use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Role {
    #[default]
    Viewer,
    User,
    Admin,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::User => write!(f, "User"),
            Role::Viewer => write!(f, "Viewer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub username: String,
    // #[serde(skip_serializing)] // Removed to allow DB serialization. TODO: Use separate DTO for API to hide hash
    pub password_hash: String,
    pub role: Role,
    pub created_at: String,
    pub last_login: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub role: Role,
    pub created_at: String,
    pub last_login: Option<String>,
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            role: user.role,
            created_at: user.created_at,
            last_login: user.last_login,
            is_active: user.is_active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_role_display() {
        assert_eq!(format!("{}", Role::Admin), "Admin");
        assert_eq!(format!("{}", Role::User), "User");
        assert_eq!(format!("{}", Role::Viewer), "Viewer");
    }

    #[test]
    fn test_role_default() {
        assert_eq!(Role::default(), Role::Viewer);
    }

    #[test]
    fn test_role_ordering() {
        assert!(Role::Viewer < Role::User);
        assert!(Role::User < Role::Admin);
    }

    #[test]
    fn test_user_response_excludes_password_hash() {
        let user = User {
            id: "user-123".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password_value".to_string(),
            role: Role::Admin,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_login: None,
            is_active: true,
        };

        let user_response = UserResponse::from(user);
        let json = serde_json::to_string(&user_response).unwrap();

        // Verify password_hash is not in the serialized output
        assert!(
            !json.contains("password_hash"),
            "password_hash should not be in UserResponse serialization"
        );
        assert!(
            !json.contains("hashed_password_value"),
            "password value should not be in UserResponse serialization"
        );

        // Verify other fields are present
        assert!(json.contains("user-123"), "id should be in UserResponse");
        assert!(
            json.contains("testuser"),
            "username should be in UserResponse"
        );
    }

    #[test]
    fn test_user_response_from_user() {
        let user = User {
            id: "user-456".to_string(),
            username: "testuser2".to_string(),
            password_hash: "another_hash".to_string(),
            role: Role::User,
            created_at: "2024-01-02T00:00:00Z".to_string(),
            last_login: Some("2024-01-03T00:00:00Z".to_string()),
            is_active: true,
        };

        let user_response = UserResponse::from(user);

        assert_eq!(user_response.id, "user-456");
        assert_eq!(user_response.username, "testuser2");
        assert_eq!(user_response.role, Role::User);
        assert_eq!(user_response.created_at, "2024-01-02T00:00:00Z");
        assert_eq!(
            user_response.last_login,
            Some("2024-01-03T00:00:00Z".to_string())
        );
        assert!(user_response.is_active);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Option<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<Role>,
    pub is_active: Option<bool>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}
