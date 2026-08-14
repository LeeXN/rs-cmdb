use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use common::entity::user::{Role, User};
use common::error::{CmdbError, CmdbResult};
use hmac::{Hmac, Mac};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use std::sync::OnceLock;

const MASTER_CLIENT_KEY_STORAGE_KEY: &str = "config:master_client_key";

/// Validate password complexity requirements.
///
/// Requirements:
/// - Minimum 12 characters
/// - At least one uppercase letter (A-Z)
/// - At least one lowercase letter (a-z)
/// - At least one number (0-9)
/// - At least one special character
pub fn validate_password_complexity(password: &str) -> CmdbResult<()> {
    // Check minimum length
    if password.len() < 12 {
        return Err(CmdbError::Validation(
            "Password must be at least 12 characters".to_string(),
        ));
    }

    // Check for uppercase letter
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(CmdbError::Validation(
            "Password must contain at least one uppercase letter (A-Z)".to_string(),
        ));
    }

    // Check for lowercase letter
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(CmdbError::Validation(
            "Password must contain at least one lowercase letter (a-z)".to_string(),
        ));
    }

    // Check for number
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(CmdbError::Validation(
            "Password must contain at least one number (0-9)".to_string(),
        ));
    }

    // Check for special character
    if !password
        .chars()
        .any(|c| c.is_ascii_punctuation() || !c.is_alphanumeric())
    {
        return Err(CmdbError::Validation(
            "Password must contain at least one special character".to_string(),
        ));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub role: Role,
    pub exp: usize,
    pub aud: String,
    pub iss: String,
    pub jti: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: usize,
    pub jti: String,
    pub typ: String, // "refresh"
}

/// Standalone hash a token/secret using Argon2.
pub fn hash_token(token: &str) -> CmdbResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(token.as_bytes(), &salt)
        .map_err(|e| CmdbError::Internal(format!("Failed to hash token: {}", e)))
        .map(|h| h.to_string())
}

/// Standalone verify a token/secret against an Argon2 hash.
pub fn verify_token(token: &str, token_hash: &str) -> CmdbResult<bool> {
    let parsed_hash = PasswordHash::new(token_hash)
        .map_err(|e| CmdbError::Internal(format!("Failed to parse token hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(token.as_bytes(), &parsed_hash)
        .is_ok())
}

static MASTER_CLIENT_KEY: OnceLock<String> = OnceLock::new();

/// Initialize the master HMAC key for client token generation.
/// Must be called once at server startup.
pub fn init_master_client_key(key: String) {
    MASTER_CLIENT_KEY.set(key).ok();
}

/// Load the agent-token signing key from durable server storage, creating it
/// only on the first startup. Generating this key on every process restart
/// invalidates all persisted agent tokens and makes already-running agents
/// fail authentication until they register again.
pub async fn init_master_client_key_from_db(db: Arc<dyn crate::db::Database>) -> CmdbResult<()> {
    let existing = db
        .get(MASTER_CLIENT_KEY_STORAGE_KEY)
        .await?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .filter(|key| key.len() >= 32);
    let key = match existing {
        Some(key) => key,
        None => {
            let key = Uuid::new_v4().to_string()
                + &Uuid::new_v4().to_string()
                + &Uuid::new_v4().to_string()
                + &Uuid::new_v4().to_string();
            db.set(MASTER_CLIENT_KEY_STORAGE_KEY, key.as_bytes())
                .await?;
            key
        }
    };
    init_master_client_key(key);
    Ok(())
}

/// Generate a deterministic client token bound to the given client_id
/// using HMAC-SHA256. This binds the token to the specific client so
/// it cannot be reused to impersonate other clients.
pub fn generate_client_token(client_id: &str) -> String {
    let key = MASTER_CLIENT_KEY
        .get()
        .expect("MASTER_CLIENT_KEY not initialized - call init_master_client_key at startup");
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC key should be valid length");
    mac.update(client_id.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Check whether a presented token was issued for `client_id` by this server.
/// HMAC verification avoids scanning every stored Argon2 hash during legacy
/// identity migration and performs the digest comparison in constant time.
pub fn client_token_matches_id(token: &str, client_id: &str) -> bool {
    let Ok(token_bytes) = hex::decode(token) else {
        return false;
    };
    let Some(key) = MASTER_CLIENT_KEY.get() else {
        return false;
    };
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(key.as_bytes()) else {
        return false;
    };
    mac.update(client_id.as_bytes());
    mac.verify_slice(&token_bytes).is_ok()
}

pub struct AuthService {
    jwt_secret: String,
    refresh_secret: String,
    aud: String,
    iss: String,
    blacklist: Option<Arc<super::token_blacklist::TokenBlacklist>>,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        // Derive the refresh-token key from the durable JWT secret instead of
        // generating it per process. Otherwise a normal server restart would
        // invalidate every still-valid refresh token.
        let mut refresh_hasher = Sha256::new();
        refresh_hasher.update(b"rs-cmdb:refresh-token:");
        refresh_hasher.update(jwt_secret.as_bytes());
        let refresh_secret = hex::encode(refresh_hasher.finalize());
        Self {
            jwt_secret,
            refresh_secret,
            aud: "rs-cmdb-api".to_string(),
            iss: "rs-cmdb".to_string(),
            blacklist: None,
        }
    }

    pub fn with_blacklist(mut self, bl: Arc<super::token_blacklist::TokenBlacklist>) -> Self {
        self.blacklist = Some(bl);
        self
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

    #[allow(dead_code)]
    pub fn generate_token(&self, user: &User) -> CmdbResult<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(30))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: expiration as usize,
            aud: self.aud.clone(),
            iss: self.iss.clone(),
            jti: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| CmdbError::Internal(format!("Failed to generate token: {}", e)))
    }

    #[allow(dead_code)]
    pub fn generate_refresh_token(&self, user: &User) -> CmdbResult<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .expect("valid timestamp")
            .timestamp();

        let claims = RefreshClaims {
            sub: user.id.clone(),
            exp: expiration as usize,
            jti: Uuid::new_v4().to_string(),
            typ: "refresh".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.refresh_secret.as_bytes()),
        )
        .map_err(|e| CmdbError::Internal(format!("Failed to generate refresh token: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> CmdbResult<Claims> {
        let mut validation = Validation::default();
        validation.set_audience(&[&self.aud]);
        validation.set_issuer(&[&self.iss]);
        validation.leeway = 10;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| {
            let msg = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token expired",
                jsonwebtoken::errors::ErrorKind::InvalidAudience => "Invalid audience",
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => "Invalid issuer",
                _ => "Invalid token",
            };
            CmdbError::Auth(msg.to_string())
        })?;

        Ok(token_data.claims)
    }

    /// Check blacklist after obtaining Claims.
    pub async fn check_blacklist(&self, jti: &str) -> CmdbResult<()> {
        if let Some(ref bl) = self.blacklist {
            if bl.is_revoked(jti).await {
                return Err(CmdbError::Auth("Token has been revoked".to_string()));
            }
        }
        Ok(())
    }

    pub fn verify_refresh_token(&self, token: &str) -> CmdbResult<RefreshClaims> {
        let mut validation = Validation::default();
        validation.leeway = 10;

        let token_data = decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(self.refresh_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| {
            let msg = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Refresh token expired",
                _ => "Invalid refresh token",
            };
            CmdbError::Auth(msg.to_string())
        })?;

        if token_data.claims.typ != "refresh" {
            return Err(CmdbError::Auth("Not a refresh token".to_string()));
        }

        Ok(token_data.claims)
    }

    /// Revoke a token by jti.
    pub async fn revoke_token(&self, jti: &str, exp: usize) {
        if let Some(ref bl) = self.blacklist {
            bl.revoke(jti, exp).await;
        }
    }

    pub fn generate_access_refresh_pair(
        &self,
        user: &User,
    ) -> CmdbResult<(String, String, usize, usize)> {
        let (access, access_exp) = {
            let exp = Utc::now()
                .checked_add_signed(Duration::minutes(30))
                .expect("valid timestamp")
                .timestamp() as usize;
            let claims = Claims {
                sub: user.id.clone(),
                username: user.username.clone(),
                role: user.role.clone(),
                exp,
                aud: self.aud.clone(),
                iss: self.iss.clone(),
                jti: Uuid::new_v4().to_string(),
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
            )
            .map_err(|e| CmdbError::Internal(format!("Failed to generate token: {}", e)))?;
            (token, exp)
        };

        let (refresh, refresh_exp) = {
            let exp = Utc::now()
                .checked_add_signed(Duration::days(7))
                .expect("valid timestamp")
                .timestamp() as usize;
            let claims = RefreshClaims {
                sub: user.id.clone(),
                exp,
                jti: Uuid::new_v4().to_string(),
                typ: "refresh".to_string(),
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(self.refresh_secret.as_bytes()),
            )
            .map_err(|e| CmdbError::Internal(format!("Failed to generate refresh token: {}", e)))?;
            (token, exp)
        };

        Ok((access, refresh, access_exp, refresh_exp))
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
            .checked_add_signed(chrono::Duration::minutes(30))
            .expect("Valid timestamp")
            .timestamp() as usize;
        assert_eq!(
            claims.exp, exp_time,
            "Token should have 30 minute expiration"
        );
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

    #[test]
    fn test_validate_password_complexity_valid_password() {
        let password = "SecureP@ssword123";
        let result = validate_password_complexity(password);
        assert!(result.is_ok(), "Valid password should pass validation");
    }

    #[test]
    fn test_validate_password_complexity_too_short() {
        let password = "Short1!";
        let result = validate_password_complexity(password);
        assert!(result.is_err(), "Password too short should fail");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("at least 12 characters"));
    }

    #[test]
    fn test_validate_password_complexity_missing_uppercase() {
        let password = "lowercase123!";
        let result = validate_password_complexity(password);
        assert!(result.is_err(), "Password without uppercase should fail");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("uppercase letter"));
    }

    #[test]
    fn test_validate_password_complexity_missing_lowercase() {
        let password = "UPPERCASE123!";
        let result = validate_password_complexity(password);
        assert!(result.is_err(), "Password without lowercase should fail");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("lowercase letter"));
    }

    #[test]
    fn test_validate_password_complexity_missing_number() {
        let password = "NoNumbersHere!";
        let result = validate_password_complexity(password);
        assert!(result.is_err(), "Password without number should fail");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("number"));
    }

    #[test]
    fn test_validate_password_complexity_missing_special() {
        let password = "NoSpecialChars123";
        let result = validate_password_complexity(password);
        assert!(
            result.is_err(),
            "Password without special character should fail"
        );
        let err = result.unwrap_err();
        assert!(err.to_string().contains("special character"));
    }

    #[test]
    fn test_hash_verify_token_roundtrip() {
        let token = "my-secret-agent-token-12345";
        let hash = hash_token(token).expect("hash should succeed");
        assert!(verify_token(token, &hash).unwrap());
        assert!(!verify_token("wrong-token", &hash).unwrap());
    }

    #[test]
    fn test_hash_token_different_tokens() {
        let h1 = hash_token("token-a").unwrap();
        let h2 = hash_token("token-b").unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_generate_client_token_deterministic() {
        init_master_client_key("test-deterministic-key".to_string());
        let token1 = generate_client_token("client-001");
        let token2 = generate_client_token("client-001");
        assert_eq!(token1, token2, "same client_id should produce same token");
    }

    #[test]
    fn test_generate_client_token_binds_to_client_id() {
        init_master_client_key("test-binding-key".to_string());
        let token_a = generate_client_token("client-A");
        let token_b = generate_client_token("client-B");
        assert_ne!(
            token_a, token_b,
            "different client_ids must produce different tokens"
        );
    }

    #[test]
    fn test_generate_client_token_length() {
        init_master_client_key("test-length-key".to_string());
        let token = generate_client_token("some-client");
        // HMAC-SHA256 output is 32 bytes → 64 hex chars
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_generate_access_refresh_pair_different_tokens() {
        let auth = AuthService::new("test-jwt-secret-for-unit-tests!!".to_string());
        let user = User {
            id: "user-001".to_string(),
            username: "test".to_string(),
            password_hash: "hash".to_string(),
            role: Role::User,
            created_at: Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };
        let (access, refresh, access_exp, refresh_exp) =
            auth.generate_access_refresh_pair(&user).unwrap();
        assert_ne!(access, refresh);
        assert!(refresh_exp > access_exp);
    }

    #[test]
    fn test_refresh_token_survives_auth_service_recreation() {
        let secret = "stable-jwt-secret-for-refresh-tests!!";
        let first = AuthService::new(secret.to_string());
        let user = User {
            id: "user-refresh".to_string(),
            username: "refresh-test".to_string(),
            password_hash: "hash".to_string(),
            role: Role::User,
            created_at: Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };
        let (_, refresh, _, _) = first.generate_access_refresh_pair(&user).unwrap();

        let recreated = AuthService::new(secret.to_string());
        let claims = recreated.verify_refresh_token(&refresh).unwrap();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.typ, "refresh");
    }

    #[tokio::test]
    async fn test_token_blacklist_revoke() {
        use crate::service::token_blacklist::TokenBlacklist;
        use crate::tests::fixtures::setup_test_db;

        let db = Arc::new(setup_test_db().unwrap());
        let bl = Arc::new(TokenBlacklist::new(db));
        let auth =
            AuthService::new("test-jwt-secret-for-unit-tests!!".to_string()).with_blacklist(bl);

        let user = User {
            id: "user-blacklist".to_string(),
            username: "blacklist-test".to_string(),
            password_hash: "hash".to_string(),
            role: Role::User,
            created_at: Utc::now().to_rfc3339(),
            last_login: None,
            is_active: true,
        };
        let (access, _refresh, access_exp, _) = auth.generate_access_refresh_pair(&user).unwrap();
        let claims = auth.verify_token(&access).unwrap();
        assert_eq!(claims.sub, "user-blacklist");

        auth.revoke_token(&claims.jti, access_exp).await;
        let err = auth.check_blacklist(&claims.jti).await.unwrap_err();
        assert!(err.to_string().contains("revoked"));
    }
}
