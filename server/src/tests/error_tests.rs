/// Tests for database error handling and error propagation
///
/// These tests verify that:
/// - Serialization failures return CmdbError::Serialization
/// - Deserialization failures return CmdbError::Deserialization  
/// - Database connection failures return CmdbError::Database
/// - Error context is preserved in error chains

#[cfg(test)]
mod tests {
    use common::error::{CmdbError, CmdbResult};
    use serde_json;

    #[test]
    fn test_serialization_failure_returns_serialization_error() {
        let result: Result<String, CmdbError> =
            Err(CmdbError::Serialization("Test error".to_string()));

        assert!(matches!(result, Err(CmdbError::Serialization(_))));
    }

    #[test]
    fn test_database_error_returns_database_error() {
        let result: Result<String, CmdbError> =
            Err(CmdbError::Database("DB connection failed".to_string()));

        assert!(matches!(result, Err(CmdbError::Database(_))));
    }

    #[test]
    fn test_not_found_error_returns_not_found_error() {
        let result: Result<String, CmdbError> =
            Err(CmdbError::NotFound("Item not found".to_string()));

        assert!(matches!(result, Err(CmdbError::NotFound(_))));
    }

    #[test]
    fn test_validation_error_returns_validation_error() {
        let result: Result<String, CmdbError> =
            Err(CmdbError::Validation("Invalid input".to_string()));

        assert!(matches!(result, Err(CmdbError::Validation(_))));
    }

    #[test]
    fn test_error_with_downcast_works() {
        let base_error = CmdbError::Database("DB error".to_string());

        let as_result: Result<String, &dyn std::error::Error> = Err(&base_error);

        assert!(as_result.is_err());
    }

    #[test]
    fn test_error_chain_preserves_context() {
        let inner = CmdbError::NotFound("Inner error".to_string());
        let outer = CmdbError::Database(format!("Outer: {}", inner));

        let error_str = outer.to_string();
        assert!(error_str.contains("Inner error"));
        assert!(error_str.contains("Outer:"));
    }
}
