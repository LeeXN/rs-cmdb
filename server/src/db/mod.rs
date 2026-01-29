pub mod redb_store;

use async_trait::async_trait;
use common::error::CmdbResult;

/// Generic database interface
///
/// This trait defines the operations that any storage implementation must support.
/// Using a trait allows us to easily swap storage implementations in the future.
#[async_trait]
pub trait Database: Send + Sync + 'static {
    /// Store a value for a given key
    async fn set(&self, key: &str, value: &[u8]) -> CmdbResult<()>;

    /// Retrieve a value for a given key
    async fn get(&self, key: &str) -> CmdbResult<Option<Vec<u8>>>;

    /// Delete a value for a given key
    async fn delete(&self, key: &str) -> CmdbResult<()>;

    /// List all keys with a given prefix
    async fn list_keys(&self, prefix: &str) -> CmdbResult<Vec<String>>;

    /// List all values with a given prefix
    async fn list_values(&self, prefix: &str) -> CmdbResult<Vec<Vec<u8>>>;

    /// List all key-value pairs with a given prefix
    async fn list_entries(&self, prefix: &str) -> CmdbResult<Vec<(String, Vec<u8>)>>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> CmdbResult<bool>;

    /// Update entries matching a prefix transactionally.
    /// The callback takes (key, value) and returns:
    /// - Some(new_value) to update
    /// - None to keep unchanged
    async fn update_all(
        &self,
        prefix: &str,
        callback: Box<dyn Fn(String, Vec<u8>) -> Option<Vec<u8>> + Send + Sync>,
    ) -> CmdbResult<()>;
}
