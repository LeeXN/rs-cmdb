//! Cached repository wrappers
//!
//! Provides cached implementations of repositories to reduce
//! database access for frequently accessed data.

use crate::cache::{CacheConfigs, CacheService, cache_service::key_builder};
use crate::repository::client_repository::ClientRepository;
use common::error::CmdbResult;
use common::models::Client;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Cached client repository wrapper
pub struct CachedClientRepository {
    inner: Arc<ClientRepository>,
    cache: CacheService<String, Option<Client>>,
}

impl CachedClientRepository {
    /// Create a new cached client repository
    pub fn new(inner: Arc<ClientRepository>, cache_configs: &CacheConfigs) -> Self {
        Self {
            inner,
            cache: CacheService::with_config(cache_configs.client_data.clone()),
        }
    }

    /// Get a client by ID (cached)
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> CmdbResult<Option<Client>> {
        let cache_key = key_builder::client(id);

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            debug!("Cache hit for client: {}", id);
            return Ok(cached);
        }

        debug!("Cache miss for client: {}, fetching from database", id);

        // Fetch from database
        let client = self.inner.get(id).await?;

        // Store in cache
        self.cache.insert(cache_key, client.clone()).await;

        Ok(client)
    }

    /// Save a client (invalidates cache)
    #[instrument(skip(self, client))]
    pub async fn save(&self, client: &Client) -> CmdbResult<()> {
        // Save to database
        self.inner.save(client).await?;

        // Invalidate cache for this client
        let cache_key = key_builder::client(&client.id);
        self.cache.invalidate(&cache_key).await;

        // Also invalidate list cache (we could optimize this with tags)
        self.cache.invalidate_all();

        Ok(())
    }

    /// Delete a client (invalidates cache)
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> CmdbResult<()> {
        // Delete from database
        self.inner.delete(id).await?;

        // Invalidate cache
        let cache_key = key_builder::client(id);
        self.cache.invalidate(&cache_key).await;

        // Invalidate list cache
        self.cache.invalidate_all();

        Ok(())
    }

    /// List all clients (not cached - this should be filtered first)
    #[instrument(skip(self))]
    pub async fn list_all(&self) -> CmdbResult<Vec<Client>> {
        // Don't cache the full list - it's too large and changes frequently
        // In a real system, you'd cache filtered views
        self.inner.list_all().await
    }

    /// Check if a client exists
    #[instrument(skip(self))]
    pub async fn exists(&self, id: &str) -> CmdbResult<bool> {
        // Use cached get to check existence
        Ok(self.get(id).await?.is_some())
    }

    /// Find client by serial number
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn find_by_serial(&self, serial: &str) -> CmdbResult<Option<Client>> {
        // This could be optimized with a separate cache key
        self.inner.find_by_serial(serial).await
    }

    /// Update client primary IP (invalidates cache)
    #[instrument(skip(self))]
    pub async fn update_primary_ip(&self, id: &str, primary_ip: &str) -> CmdbResult<()> {
        self.inner.update_primary_ip(id, primary_ip).await?;

        let cache_key = key_builder::client(id);
        self.cache.invalidate(&cache_key).await;

        Ok(())
    }

    /// Update client last seen timestamp
    #[instrument(skip(self))]
    pub async fn update_last_seen(&self, id: &str) -> CmdbResult<()> {
        self.inner.update_last_seen(id).await?;

        // Invalidate cache for this client
        let cache_key = key_builder::client(id);
        self.cache.invalidate(&cache_key).await;

        Ok(())
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub fn cache_stats(&self) -> crate::cache::cache_service::CacheStats {
        self.cache.stats()
    }

    /// Invalidate all cache entries
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::models::Client;

    fn test_client(id: &str, hostname: &str) -> Client {
        Client {
            id: id.to_string(),
            hostname: hostname.to_string(),
            ip_address: "192.168.1.1".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_cached_repository_get() {
        let db = setup_test_db().unwrap();
        let inner = Arc::new(ClientRepository::new(Arc::new(db)));
        let cached = CachedClientRepository::new(inner, &CacheConfigs::default());

        // Nonexistent client returns None
        let result = cached.get("nonexistent").await.unwrap();
        assert!(result.is_none());

        // Save and retrieve
        let client = test_client("test-001", "test-host");
        cached.save(&client).await.unwrap();

        let result = cached.get("test-001").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().hostname, "test-host");
    }

    #[tokio::test]
    async fn test_cached_repository_invalidation() {
        let db = setup_test_db().unwrap();
        let inner = Arc::new(ClientRepository::new(Arc::new(db)));
        let cached = CachedClientRepository::new(inner.clone(), &CacheConfigs::default());

        let client = test_client("test-002", "original");
        cached.save(&client).await.unwrap();

        // Verify save works
        let result = cached.get("test-002").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().hostname, "original");

        // Update directly in inner repo (bypass cache)
        let updated = test_client("test-002", "updated");
        inner.save(&updated).await.unwrap();

        // Invalidate cache
        cached.invalidate_all();

        // Next get should fetch fresh from DB
        let result = cached.get("test-002").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().hostname, "updated");
    }

    #[tokio::test]
    async fn test_cached_repository_delete() {
        let db = setup_test_db().unwrap();
        let inner = Arc::new(ClientRepository::new(Arc::new(db)));
        let cached = CachedClientRepository::new(inner, &CacheConfigs::default());

        let client = test_client("test-003", "to-delete");
        cached.save(&client).await.unwrap();

        // Verify exists
        let result = cached.get("test-003").await.unwrap();
        assert!(result.is_some());

        // Delete
        cached.delete("test-003").await.unwrap();

        // Should return None after delete
        let result = cached.get("test-003").await.unwrap();
        assert!(result.is_none());
    }
}
