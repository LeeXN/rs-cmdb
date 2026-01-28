//! Test fixtures and utilities for client testing

/// Test data directory path constant
pub const TEST_DATA_DIR: &str = "tests/testdata";

/// Returns the path to test data files
///
/// # Example
/// ```
/// let cpuinfo_path = test_data_path("proc_cpuinfo.txt");
/// ```
pub fn test_data_path(file: &str) -> String {
    std::path::Path::new(TEST_DATA_DIR)
        .join(file)
        .to_str()
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_data_path() {
        let path = test_data_path("test.txt");
        assert!(path.contains("tests/testdata"));
        assert!(path.contains("test.txt"));
    }
}
