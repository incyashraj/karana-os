// Kāraṇa OS - Search Result Cache
// Phase 3 & 5: Performance optimization with LRU caching

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Cached search result with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub answer: String,
    pub confidence: f32,
    pub cached_at: u64,
    pub ttl_seconds: u64,
}

impl CachedResult {
    pub fn new(answer: String, confidence: f32, ttl_seconds: u64) -> Self {
        Self {
            answer,
            confidence,
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.cached_at > self.ttl_seconds
    }
}

/// LRU cache for search results and LLM responses
pub struct SearchCache {
    cache: Mutex<LruCache<String, CachedResult>>,
    hits: Mutex<usize>,
    misses: Mutex<usize>,
}

impl SearchCache {
    /// Create new cache with capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap())),
            hits: Mutex::new(0),
            misses: Mutex::new(0),
        }
    }

    /// Get cached result for query
    pub fn get(&self, query: &str) -> Option<CachedResult> {
        let mut cache = self.cache.lock().unwrap();
        
        if let Some(result) = cache.get(query) {
            if !result.is_expired() {
                *self.hits.lock().unwrap() += 1;
                log::debug!("[SearchCache] HIT: {}", query);
                return Some(result.clone());
            } else {
                // Remove expired entry
                cache.pop(query);
                log::debug!("[SearchCache] EXPIRED: {}", query);
            }
        }
        
        *self.misses.lock().unwrap() += 1;
        log::debug!("[SearchCache] MISS: {}", query);
        None
    }

    /// Cache a result
    pub fn put(&self, query: String, result: CachedResult) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(query.clone(), result);
        log::debug!("[SearchCache] CACHED: {}", query);
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
        log::info!("[SearchCache] Cleared");
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            total_requests: total,
            hit_rate,
            size: self.cache.lock().unwrap().len(),
        }
    }

    /// Preload common queries
    pub fn preload(&self, queries: Vec<(String, CachedResult)>) {
        let mut cache = self.cache.lock().unwrap();
        for (query, result) in queries {
            cache.put(query, result);
        }
        log::info!("[SearchCache] Preloaded {} queries", cache.len());
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub total_requests: usize,
    pub hit_rate: f32,
    pub size: usize,
}

impl Default for SearchCache {
    fn default() -> Self {
        // Default: 1000 cached queries
        Self::new(1000)
    }
}

/// Embedding cache for frequently queried texts
pub struct EmbeddingCache {
    cache: Mutex<LruCache<String, Vec<f32>>>,
}

impl EmbeddingCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap())),
        }
    }

    pub fn get(&self, text: &str) -> Option<Vec<f32>> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(text).cloned()
    }

    pub fn put(&self, text: String, embedding: Vec<f32>) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(text, embedding);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

impl Default for EmbeddingCache {
    fn default() -> Self {
        Self::new(500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_cache() {
        let cache = SearchCache::new(10);

        // Cache a result
        let result = CachedResult::new("Test answer".to_string(), 0.9, 3600);
        cache.put("test query".to_string(), result.clone());

        // Retrieve it
        let cached = cache.get("test query");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().answer, "Test answer");

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 100.0);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = SearchCache::new(10);

        // Cache with 0 TTL (already expired)
        let result = CachedResult::new("Expired".to_string(), 0.9, 0);
        cache.put("expire_test".to_string(), result);

        // Sleep to ensure expiration
        std::thread::sleep(Duration::from_millis(10));

        // Should not retrieve expired entry
        let cached = cache.get("expire_test");
        assert!(cached.is_none());
    }

    #[test]
    fn test_embedding_cache() {
        let cache = EmbeddingCache::new(5);

        let embedding = vec![0.1, 0.2, 0.3];
        cache.put("test text".to_string(), embedding.clone());

        let retrieved = cache.get("test text");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), embedding);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = SearchCache::new(2);

        cache.put("query1".to_string(), CachedResult::new("a1".to_string(), 0.9, 3600));
        cache.put("query2".to_string(), CachedResult::new("a2".to_string(), 0.9, 3600));
        cache.put("query3".to_string(), CachedResult::new("a3".to_string(), 0.9, 3600));

        // query1 should be evicted (LRU)
        assert!(cache.get("query1").is_none());
        assert!(cache.get("query2").is_some());
        assert!(cache.get("query3").is_some());
    }
}
