//! Cache layer for performance optimization
//!
//! This module provides caching functionality using Moka,
//! a high-performance concurrent cache library.

pub mod cache_service;
pub mod cached_repository;

pub use cache_service::{CacheConfig, CacheConfigs, CacheService};
pub use cached_repository::CachedClientRepository;
