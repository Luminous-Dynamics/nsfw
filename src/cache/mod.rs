/// Simple in-memory cache for search results
///
/// Caches search results to avoid re-running expensive WSL2 commands
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::nix_ops::types::SearchResult;

/// Cache entry with expiration
#[derive(Clone)]
struct CacheEntry {
    results: Vec<SearchResult>,
    created_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn new(results: Vec<SearchResult>, ttl: Duration) -> Self {
        Self {
            results,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Global search cache
static SEARCH_CACHE: Lazy<Mutex<HashMap<String, CacheEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Search result cache manager
pub struct SearchCache;

impl SearchCache {
    /// Default TTL for cached results (5 minutes)
    const DEFAULT_TTL: Duration = Duration::from_secs(300);

    /// Get cached search results if available and not expired
    pub fn get(query: &str, limit: usize) -> Option<Vec<SearchResult>> {
        let cache = SEARCH_CACHE.lock().ok()?;
        let key = Self::make_key(query, limit);

        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                log::debug!("Cache hit for query '{}' (limit: {})", query, limit);
                return Some(entry.results.clone());
            } else {
                log::debug!("Cache expired for query '{}' (limit: {})", query, limit);
            }
        }

        None
    }

    /// Store search results in cache
    pub fn put(query: &str, limit: usize, results: Vec<SearchResult>) {
        if let Ok(mut cache) = SEARCH_CACHE.lock() {
            let key = Self::make_key(query, limit);
            let entry = CacheEntry::new(results, Self::DEFAULT_TTL);
            cache.insert(key, entry);
            log::debug!("Cached results for query '{}' (limit: {})", query, limit);
        }
    }

    /// Clear all cached results
    pub fn clear() {
        if let Ok(mut cache) = SEARCH_CACHE.lock() {
            cache.clear();
            log::debug!("Search cache cleared");
        }
    }

    /// Remove expired entries from cache
    pub fn cleanup_expired() {
        if let Ok(mut cache) = SEARCH_CACHE.lock() {
            cache.retain(|_, entry| !entry.is_expired());
            log::debug!("Cleaned up expired cache entries");
        }
    }

    /// Get cache statistics
    pub fn stats() -> (usize, usize) {
        if let Ok(cache) = SEARCH_CACHE.lock() {
            let total = cache.len();
            let expired = cache.values().filter(|e| e.is_expired()).count();
            (total, expired)
        } else {
            (0, 0)
        }
    }

    /// Make cache key from query and limit
    fn make_key(query: &str, limit: usize) -> String {
        format!("{}:{}", query.to_lowercase(), limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_and_get() {
        SearchCache::clear();

        let results = vec![SearchResult {
            pname: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test package".to_string(),
        }];

        SearchCache::put("firefox", 10, results.clone());

        let cached = SearchCache::get("firefox", 10);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        SearchCache::clear();

        let cached = SearchCache::get("nonexistent", 10);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_key_case_insensitive() {
        SearchCache::clear();

        let results = vec![SearchResult {
            pname: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test package".to_string(),
        }];

        SearchCache::put("Firefox", 10, results.clone());

        let cached = SearchCache::get("firefox", 10);
        assert!(cached.is_some());
    }

    #[test]
    fn test_cache_stats() {
        SearchCache::clear();

        let results = vec![SearchResult {
            pname: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test package".to_string(),
        }];

        SearchCache::put("vim", 10, results.clone());
        SearchCache::put("emacs", 10, results.clone());

        let (total, _expired) = SearchCache::stats();
        assert_eq!(total, 2);
    }
}
