use config::{Config, ConfigError, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Error type for configuration validation
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    InvalidJwtSecret(String),
    JwtSecretTooShort,
    JwtSecretEmpty,
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigValidationError::InvalidJwtSecret(msg) => {
                write!(f, "Invalid JWT secret: {}", msg)
            }
            ConfigValidationError::JwtSecretTooShort => {
                write!(f, "JWT secret must be at least 32 characters")
            }
            ConfigValidationError::JwtSecretEmpty => write!(f, "JWT secret cannot be empty"),
        }
    }
}

impl std::error::Error for ConfigValidationError {}

/// Validate JWT secret configuration
///
/// Ensures the JWT secret:
/// 1. Is not the default value "change_me_in_production"
/// 2. Is not empty
/// 3. Is at least 32 characters long
pub fn validate_jwt_secret(secret: &str) -> Result<(), ConfigValidationError> {
    // Check for default value
    if secret == "change_me_in_production" {
        return Err(ConfigValidationError::InvalidJwtSecret(
            "JWT secret must be changed from default value 'change_me_in_production'".to_string(),
        ));
    }

    // Check for empty string
    if secret.is_empty() {
        return Err(ConfigValidationError::JwtSecretEmpty);
    }

    // Check minimum length
    if secret.len() < 32 {
        return Err(ConfigValidationError::JwtSecretTooShort);
    }

    Ok(())
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Message queue configuration
    pub queue: QueueConfig,
    /// Poll interval for clients (in seconds)
    pub poll_interval: u64,
    /// Client timeout (in seconds)
    pub client_timeout: u64,
    /// Log level
    pub log_level: String,
    /// Enable TLS
    pub enable_tls: bool,
    /// TLS certificate path
    pub tls_cert: Option<String>,
    /// TLS key path
    pub tls_key: Option<String>,
    /// JWT Secret
    pub jwt_secret: String,
    /// Component missing grace period (in hours)
    pub component_missing_grace_period_hours: u64,
    /// SSH known_hosts file path
    pub ssh_known_hosts_file: Option<String>,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Database type
    pub db_type: String,
    /// Database path
    pub path: String,
}

/// Message queue configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueConfig {
    /// Queue type
    pub queue_type: String,
    /// Queue capacity
    pub capacity: usize,
}

// Static configuration instance
static CONFIG: Lazy<ServerConfig> = Lazy::new(|| {
    match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            // Use default configuration
            default_config()
        }
    }
});

/// Get the server configuration
pub fn get_config() -> &'static ServerConfig {
    &CONFIG
}

/// Load configuration from files and environment
fn load_config() -> Result<ServerConfig, ConfigError> {
    let defaults = default_config();

    // Set default configuration sources
    let mut builder = Config::builder()
        .set_default("host", defaults.host)?
        .set_default("port", defaults.port)?
        .set_default("database.db_type", defaults.database.db_type)?
        .set_default("database.path", defaults.database.path)?
        .set_default("queue.queue_type", defaults.queue.queue_type)?
        .set_default("queue.capacity", defaults.queue.capacity as i64)?
        .set_default("poll_interval", defaults.poll_interval)?
        .set_default("client_timeout", defaults.client_timeout)?
        .set_default("log_level", defaults.log_level)?
        .set_default("enable_tls", defaults.enable_tls)?
        .set_default("jwt_secret", defaults.jwt_secret)?
        .set_default(
            "component_missing_grace_period_hours",
            defaults.component_missing_grace_period_hours,
        )?;

    if let Some(cert) = defaults.tls_cert {
        builder = builder.set_default("tls_cert", cert)?;
    }
    if let Some(key) = defaults.tls_key {
        builder = builder.set_default("tls_key", key)?;
    }
    if let Some(known_hosts) = defaults.ssh_known_hosts_file {
        builder = builder.set_default("ssh_known_hosts_file", known_hosts)?;
    }

    builder = builder
        .add_source(File::from(PathBuf::from("config/default.toml")).required(false))
        .add_source(File::from(PathBuf::from("config/local.toml")).required(false));

    // Add environment-specific configuration
    if let Ok(env) = env::var("RUN_ENV") {
        builder = builder
            .add_source(File::from(PathBuf::from(format!("config/{}.toml", env))).required(false));
    }

    // Add environment variables with prefix "CMDB_" - these take precedence
    builder = builder.add_source(config::Environment::with_prefix("CMDB").separator("__"));

    // Explicitly override JWT secret from env to avoid separator ambiguity
    if let Ok(jwt_secret) = env::var("CMDB_JWT_SECRET") {
        builder = builder.set_override("jwt_secret", jwt_secret)?;
    }

    // Build the configuration
    let config: ServerConfig = builder.build()?.try_deserialize()?;

    Ok(config)
}

/// Default configuration
fn default_config() -> ServerConfig {
    ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        database: DatabaseConfig {
            db_type: "rocksdb".to_string(),
            path: "data/cmdb.db".to_string(),
        },
        queue: QueueConfig {
            queue_type: "flume".to_string(),
            capacity: 1000,
        },
        poll_interval: 300,   // 5 minutes
        client_timeout: 3600, // 1 hour
        log_level: "info".to_string(),
        enable_tls: false,
        tls_cert: None,
        tls_key: None,
        jwt_secret: "change_me_in_production".to_string(),
        component_missing_grace_period_hours: 24,
        ssh_known_hosts_file: Some("/etc/cmdb/ssh_known_hosts".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.database.db_type, "rocksdb");
        assert_eq!(config.queue.capacity, 1000);
        assert_eq!(config.poll_interval, 300);
        assert_eq!(config.log_level, "info");
        assert!(!config.enable_tls);
        assert_eq!(config.component_missing_grace_period_hours, 24);
    }

    #[test]
    fn test_validate_jwt_secret_default_value_rejected() {
        let result = validate_jwt_secret("change_me_in_production");
        assert!(result.is_err(), "Default JWT secret should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.to_string()
                .contains("must be changed from default value")
        );
    }

    #[test]
    fn test_validate_jwt_secret_empty_rejected() {
        let result = validate_jwt_secret("");
        assert!(result.is_err(), "Empty JWT secret should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_jwt_secret_too_short_rejected() {
        let result = validate_jwt_secret("short");
        assert!(result.is_err(), "Short JWT secret should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("at least 32 characters"));
    }

    #[test]
    fn test_validate_jwt_secret_valid_secret_accepted() {
        let result = validate_jwt_secret("this-is-a-valid-secret-that-is-at-least-32-chars-long");
        assert!(result.is_ok(), "Valid JWT secret should be accepted");
    }

    #[test]
    fn test_validate_jwt_secret_exactly_32_characters() {
        let result = validate_jwt_secret("12345678901234567890123456789012");
        assert!(result.is_ok(), "32-character JWT secret should be accepted");
    }

    #[test]
    fn test_validate_jwt_secret_31_characters_rejected() {
        let result = validate_jwt_secret("1234567890123456789012345678901");
        assert!(
            result.is_err(),
            "31-character JWT secret should be rejected"
        );
    }
}
