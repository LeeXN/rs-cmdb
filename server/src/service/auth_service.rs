use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use common::entity::user::{Role, User};
use common::error::{CmdbError, CmdbResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub role: Role,
    pub exp: usize,
}

pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn hash_password(&self, password: &str) -> CmdbResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CmdbError::Internal(format!("Failed to hash password: {}", e)))?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> CmdbResult<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| CmdbError::Internal(format!("Failed to parse password hash: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn generate_token(&self, user: &User) -> CmdbResult<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| CmdbError::Internal(format!("Failed to generate token: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> CmdbResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| CmdbError::Auth("Invalid token".to_string()))?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::entity::user::Role;

    #[test]
    fn test_password_hashing() {
        let auth_service = AuthService::new("test_secret".to_string());
        let password = "my_secure_password";

        // Test hashing
        let hash = auth_service
            .hash_password(password)
            .expect("Failed to hash password");
        assert!(!hash.is_empty());
        assert_ne!(hash, password);

        // Test verification
        let is_valid = auth_service
            .verify_password(password, &hash)
            .expect("Failed to verify password");
        assert!(is_valid);

        // Test invalid password
        let is_valid = auth_service
            .verify_password("wrong_password", &hash)
            .expect("Failed to verify password");
        assert!(!is_valid);
    }

    #[test]
    fn test_jwt_token_generation_and_verification() {
        let secret = "test_secret_key_12345";
        let auth_service = AuthService::new(secret.to_string());

        let user = User {
            id: "user-123".to_string(),
            username: "testuser".to_string(),
            password_hash: "hash".to_string(),
            role: Role::Admin,
            created_at: Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };

        let token = auth_service
            .generate_token(&user)
            .expect("Failed to generate token");
        assert!(!token.is_empty());

        let claims = auth_service
            .verify_token(&token)
            .expect("Failed to verify token");

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.role, user.role);
    }

    #[test]
    fn test_verify_password_with_correct_password() {
        let secret = "test_secret";
        let auth_service = AuthService::new(secret.to_string());

        let password = "correct_password";
        let password_hash = auth_service
            .hash_password(password)
            .expect("Failed to hash password");

        let is_valid = auth_service
            .verify_password(password, &password_hash)
            .expect("Failed to verify password");

        assert!(is_valid, "Password verification should succeed");
    }

    #[test]
    fn test_verify_password_with_incorrect_password() {
        let secret = "test_secret";
        let auth_service = AuthService::new(secret.to_string());

        let correct_password = "correct_password";
        let password_hash = auth_service
            .hash_password(correct_password)
            .expect("Failed to hash password");

        let is_valid = auth_service
            .verify_password("wrong_password", &password_hash)
            .expect("Failed to verify password");

        assert!(
            !is_valid,
            "Password verification should fail with wrong password"
        );
    }

    #[test]
    fn test_generate_token_includes_correct_claims() {
        let secret = "test_secret_key_12345";
        let auth_service = AuthService::new(secret.to_string());

        let user = User {
            id: "user-456".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password".to_string(),
            role: Role::User,
            created_at: Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };

        let token = auth_service
            .generate_token(&user)
            .expect("Failed to generate token");

        let claims = auth_service
            .verify_token(&token)
            .expect("Failed to verify token");

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.role, user.role);
    }

    #[test]
    fn test_generate_token_sets_expiration() {
        let secret = "test_secret_key_12345";
        let auth_service = AuthService::new(secret.to_string());

        let user = User {
            id: "user-789".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password".to_string(),
            role: Role::Admin,
            created_at: Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };

        let token = auth_service
            .generate_token(&user)
            .expect("Failed to generate token");

        let claims = auth_service
            .verify_token(&token)
            .expect("Failed to verify token");

        let exp_time = Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("Valid timestamp")
            .timestamp() as usize;
        assert_eq!(claims.exp, exp_time, "Token should have 24 hour expiration");
    }

    #[test]
    fn test_verify_token_with_valid_token() {
        let secret = "test_secret_key_12345";
        let auth_service = AuthService::new(secret.to_string());

        let user = User {
            id: "user-valid".to_string(),
            username: "testuser".to_string(),
            password_hash: "hashed_password".to_string(),
            role: Role::Admin,
            created_at: Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };

        let token = auth_service
            .generate_token(&user)
            .expect("Failed to generate token");
        let result = auth_service.verify_token(&token);

        assert!(result.is_ok(), "Valid token verification should succeed");

        let claims = result.unwrap();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.role, user.role);
    }

    #[test]
    fn test_verify_token_with_invalid_token() {
        let secret = "test_secret_key_12345";
        let auth_service = AuthService::new(secret.to_string());

        let result = auth_service.verify_token("malformed_token");
        assert!(result.is_err(), "Malformed token should fail");
    }

    #[test]
    fn test_invalid_token() {
        let auth_service = AuthService::new("test_secret".to_string());
        let result = auth_service.verify_token("invalid.token.string");
        assert!(result.is_err());
    }
}
