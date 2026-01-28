//! Cached repository wrappers
//!
//! Provides cached implementations of repositories to reduce
//! database access for frequently accessed data.

use crate::cache::{cache_service::key_builder, CacheConfigs, CacheService};
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
    #[instrument(skip(self))]
    pub async fn find_by_serial(&self, serial: &str) -> CmdbResult<Option<Client>> {
        // This could be optimized with a separate cache key
        self.inner.find_by_serial(serial).await
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
    pub fn cache_stats(&self) -> crate::cache::cache_service::CacheStats {
        self.cache.stats()
    }

    /// Invalidate all cache entries
    #[instrument(skip(self))]
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a proper test database setup
    // For now, they serve as documentation of expected behavior

    #[tokio::test]
    async fn test_cached_repository_get() {
        // This test would need a real ClientRepository
        // let inner = Arc::new(ClientRepository::new(...));
        // let cached = CachedClientRepository::new(inner, &CacheConfigs::default());
        //
        // // First call should hit database
        // let result1 = cached.get("test-id").await;
        //
        // // Second call should hit cache
        // let result2 = cached.get("test-id").await;
        //
        // assert_eq!(result1, result2);
    }

    #[tokio::test]
    async fn test_cached_repository_invalidation() {
        // Test that save/delete properly invalidate cache
    }
}
