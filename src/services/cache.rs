// src/services/cache.rs
// DOCUMENTATION: Simple in-memory cache for Google Places API responses
// PURPOSE: Reduce API calls by caching search results

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Cache entry with expiration
#[derive(Clone, Debug)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Simple in-memory cache with TTL
/// DOCUMENTATION: Thread-safe cache for API responses
pub struct PlacesCache {
    store: Arc<RwLock<HashMap<String, CacheEntry<String>>>>,
    default_ttl: Duration,
}

impl PlacesCache {
    /// Create new cache with default TTL
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Generate cache key from search parameters
    pub fn generate_key(
        lat: f64,
        lon: f64,
        radius: u32,
        place_type: Option<&str>,
        keyword: Option<&str>,
    ) -> String {
        format!(
            "search:{}:{}:{}:{}:{}",
            (lat * 10000.0).round() as i64, // Round to ~10m precision
            (lon * 10000.0).round() as i64,
            radius,
            place_type.unwrap_or("all"),
            keyword.unwrap_or("")
        )
    }

    /// Get cached value
    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        
        if let Some(entry) = store.get(key) {
            if !entry.is_expired() {
                log::debug!("Cache HIT for key: {}", key);
                return Some(entry.data.clone());
            } else {
                log::debug!("Cache EXPIRED for key: {}", key);
            }
        } else {
            log::debug!("Cache MISS for key: {}", key);
        }
        
        None
    }

    /// Set cached value with default TTL
    pub async fn set(&self, key: String, value: String) {
        self.set_with_ttl(key, value, self.default_ttl).await;
    }

    /// Set cached value with custom TTL
    pub async fn set_with_ttl(&self, key: String, value: String, ttl: Duration) {
        let mut store = self.store.write().await;
        store.insert(key.clone(), CacheEntry::new(value, ttl));
        log::debug!("Cache SET for key: {} (TTL: {}s)", key, ttl.as_secs());
    }

    /// Clear expired entries
    pub async fn cleanup(&self) {
        let mut store = self.store.write().await;
        let before_count = store.len();
        store.retain(|_, entry| !entry.is_expired());
        let after_count = store.len();
        
        if before_count > after_count {
            log::info!(
                "Cache cleanup: removed {} expired entries ({} remaining)",
                before_count - after_count,
                after_count
            );
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let store = self.store.read().await;
        let total = store.len();
        let expired = store.values().filter(|e| e.is_expired()).count();
        
        CacheStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        let count = store.len();
        store.clear();
        log::info!("Cache cleared: {} entries removed", count);
    }
}

/// Cache statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

/// Start background cleanup task
/// DOCUMENTATION: Periodically removes expired entries
pub fn start_cleanup_task(cache: Arc<PlacesCache>, interval_seconds: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
        
        loop {
            interval.tick().await;
            cache.cleanup().await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = PlacesCache::new(60);
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value.clone()).await;
        let result = cache.get(&key).await;

        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = PlacesCache::new(1); // 1 second TTL
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value.clone()).await;
        
        // Should exist immediately
        assert!(cache.get(&key).await.is_some());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should be expired
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn test_generate_key() {
        let key1 = PlacesCache::generate_key(40.4168, -3.7038, 1000, Some("restaurant"), None);
        let key2 = PlacesCache::generate_key(40.4168, -3.7038, 1000, Some("restaurant"), None);
        let key3 = PlacesCache::generate_key(40.4169, -3.7038, 1000, Some("restaurant"), None);

        assert_eq!(key1, key2); // Same coordinates should generate same key
        assert_ne!(key1, key3); // Different coordinates should generate different key
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache = PlacesCache::new(1);
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        cache.cleanup().await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.active_entries, 0);
    }
}
