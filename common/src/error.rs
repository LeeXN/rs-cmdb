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

    /// Returns a user-safe sanitized error message (no internal details)
    pub fn to_user_message(&self) -> String {
        match self {
            CmdbError::Database(_) => "A database error occurred".into(),
            CmdbError::Client(msg) => format!("Invalid request: {}", msg),
            CmdbError::Server(_) => "An internal server error occurred".into(),
            CmdbError::Serialization(_) => "Failed to process data".into(),
            CmdbError::Network(_) => "A network error occurred".into(),
            CmdbError::NotFound(msg) => msg.clone(),
            CmdbError::Validation(msg) => msg.clone(),
            CmdbError::Auth(_) => "Authentication failed".into(),
            CmdbError::Forbidden(msg) => msg.clone(),
            CmdbError::Internal(_) => "An internal error occurred".into(),
            CmdbError::Other(msg) => msg.clone(),
        }
    }

    /// Log the full error and return a user-safe message
    pub fn log_and_user_message(&self) -> String {
        let msg = self.to_user_message();
        tracing::error!(error = %self, "API error");
        msg
    }
}

/// Result type for CMDB operations
pub type CmdbResult<T> = Result<T, CmdbError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_all_variants() {
        let cases = vec![
            (CmdbError::Database("db err".into()), "Database error: db err"),
            (CmdbError::Client("bad".into()), "Client error: bad"),
            (CmdbError::Server("fail".into()), "Server error: fail"),
            (CmdbError::Serialization("json".into()), "Serialization error: json"),
            (CmdbError::Network("timeout".into()), "Network error: timeout"),
            (CmdbError::NotFound("user".into()), "Not found: user"),
            (CmdbError::Validation("invalid".into()), "Validation error: invalid"),
            (CmdbError::Auth("bad token".into()), "Authentication error: bad token"),
            (CmdbError::Forbidden("no access".into()), "Authorization error: no access"),
            (CmdbError::Internal("oops".into()), "Internal error: oops"),
            (CmdbError::Other("generic".into()), "generic"),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_debug_all_variants() {
        let err = CmdbError::Database("test".into());
        assert!(format!("{:?}", err).contains("Database"));
    }

    #[test]
    fn test_to_user_message() {
        assert_eq!(
            CmdbError::Database("".into()).to_user_message(),
            "A database error occurred"
        );
        assert_eq!(
            CmdbError::Client("bad request".into()).to_user_message(),
            "Invalid request: bad request"
        );
        assert_eq!(
            CmdbError::Server("".into()).to_user_message(),
            "An internal server error occurred"
        );
        assert_eq!(
            CmdbError::Serialization("".into()).to_user_message(),
            "Failed to process data"
        );
        assert_eq!(
            CmdbError::Network("".into()).to_user_message(),
            "A network error occurred"
        );
        assert_eq!(
            CmdbError::NotFound("user not found".into()).to_user_message(),
            "user not found"
        );
        assert_eq!(
            CmdbError::Validation("invalid input".into()).to_user_message(),
            "invalid input"
        );
        assert_eq!(
            CmdbError::Auth("".into()).to_user_message(),
            "Authentication failed"
        );
        assert_eq!(
            CmdbError::Forbidden("no access".into()).to_user_message(),
            "no access"
        );
        assert_eq!(
            CmdbError::Internal("".into()).to_user_message(),
            "An internal error occurred"
        );
        assert_eq!(
            CmdbError::Other("something".into()).to_user_message(),
            "something"
        );
    }

    #[test]
    fn test_status_code() {
        assert_eq!(CmdbError::NotFound("".into()).status_code(), 404);
        assert_eq!(CmdbError::Validation("".into()).status_code(), 400);
        assert_eq!(CmdbError::Client("".into()).status_code(), 400);
        assert_eq!(CmdbError::Auth("".into()).status_code(), 401);
        assert_eq!(CmdbError::Forbidden("".into()).status_code(), 403);
        assert_eq!(CmdbError::Network("".into()).status_code(), 503);
        assert_eq!(CmdbError::Serialization("".into()).status_code(), 422);
        assert_eq!(CmdbError::Database("".into()).status_code(), 500);
        assert_eq!(CmdbError::Server("".into()).status_code(), 500);
        assert_eq!(CmdbError::Internal("".into()).status_code(), 500);
        assert_eq!(CmdbError::Other("".into()).status_code(), 500);
    }
}
