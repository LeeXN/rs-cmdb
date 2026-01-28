use thiserror::Error;

/// Common error types for the CMDB system
#[derive(Error, Debug)]
pub enum CmdbError {
    /// Database operation error
    #[error("Database error: {0}")]
    Database(String),

    /// Client error
    #[error("Client error: {0}")]
    Client(String),

    /// Server error
    #[error("Server error: {0}")]
    Server(String),

    /// Serialization/Deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    Forbidden(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl CmdbError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            CmdbError::NotFound(_) => 404,
            CmdbError::Validation(_) => 400,
            CmdbError::Client(_) => 400,
            CmdbError::Auth(_) => 401,
            CmdbError::Forbidden(_) => 403,
            CmdbError::Network(_) => 503,
            CmdbError::Serialization(_) => 422,
            _ => 500,
        }
    }
}

/// Result type for CMDB operations
pub type CmdbResult<T> = Result<T, CmdbError>;
