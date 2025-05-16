use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
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

impl Default for Role {
    fn default() -> Self {
        Role::Viewer
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
