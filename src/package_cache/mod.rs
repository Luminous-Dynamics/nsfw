/// Dynamic package cache for fast searches
///
/// This module provides a local SQLite database cache of nixpkgs packages.
/// The cache is built incrementally and updated automatically.
///
/// Features:
/// - Fast local searches (< 10ms)
/// - Automatic background updates
/// - Smart prioritization (popular packages first)
/// - Learning from user searches
mod builder;

pub use builder::CacheBuilder;

use rusqlite::{Connection, params, Result as SqlResult};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use log::{debug, info};

/// Cached package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub attr_path: String,
    pub last_updated: i64,  // Unix timestamp
    pub search_count: i32,  // Popularity tracking
}

/// Package cache manager
pub struct PackageCache {
    db_path: PathBuf,
}

impl PackageCache {
    /// Create a new package cache
    pub fn new() -> Result<Self> {
        let cache_dir = Self::cache_directory()?;
        std::fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        let db_path = cache_dir.join("packages.db");
        debug!("Package cache database: {}", db_path.display());

        Ok(Self { db_path })
    }

    /// Get the cache directory path
    fn cache_directory() -> Result<PathBuf> {
        // Use ~/.cache/nsfw on Linux/Mac, AppData on Windows
        let cache_dir = if cfg!(target_os = "windows") {
            dirs::cache_dir()
                .context("Could not find cache directory")?
                .join("nsfw")
        } else {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()))
                .join(".cache")
                .join("nsfw")
        };

        Ok(cache_dir)
    }

    /// Initialize the database schema
    pub fn initialize(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS packages (
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT NOT NULL,
                attr_path TEXT PRIMARY KEY,
                last_updated INTEGER NOT NULL,
                search_count INTEGER DEFAULT 0
            )",
            [],
        ).context("Failed to create packages table")?;

        // Create indexes for fast searches
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_name ON packages(name)",
            [],
        ).context("Failed to create name index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_search_count ON packages(search_count DESC)",
            [],
        ).context("Failed to create popularity index")?;

        // Metadata table for cache state
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        ).context("Failed to create metadata table")?;

        info!("Package cache database initialized");
        Ok(())
    }

    /// Search packages in the cache
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<CachedPackage>> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let search_pattern = format!("%{}%", query.to_lowercase());

        let mut stmt = conn.prepare(
            "SELECT name, version, description, attr_path, last_updated, search_count
             FROM packages
             WHERE LOWER(name) LIKE ?1 OR LOWER(description) LIKE ?1
             ORDER BY search_count DESC, name ASC
             LIMIT ?2"
        ).context("Failed to prepare search query")?;

        let packages = stmt.query_map(params![search_pattern, limit as i32], |row| {
            Ok(CachedPackage {
                name: row.get(0)?,
                version: row.get(1)?,
                description: row.get(2)?,
                attr_path: row.get(3)?,
                last_updated: row.get(4)?,
                search_count: row.get(5)?,
            })
        }).context("Failed to execute search")?
          .collect::<SqlResult<Vec<_>>>()
          .context("Failed to collect results")?;

        debug!("Cache search for '{}': found {} results", query, packages.len());

        // Increment search count for found packages
        if !packages.is_empty() {
            self.increment_search_counts(&packages)?;
        }

        Ok(packages)
    }

    /// Increment search counts for packages (popularity tracking)
    fn increment_search_counts(&self, packages: &[CachedPackage]) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let attr_paths: Vec<&str> = packages.iter().map(|p| p.attr_path.as_str()).collect();
        let placeholders = vec!["?"; attr_paths.len()].join(",");

        let query = format!(
            "UPDATE packages SET search_count = search_count + 1 WHERE attr_path IN ({})",
            placeholders
        );

        let mut stmt = conn.prepare(&query)
            .context("Failed to prepare update query")?;

        stmt.execute(rusqlite::params_from_iter(attr_paths))
            .context("Failed to update search counts")?;

        Ok(())
    }

    /// Add or update packages in the cache
    pub fn upsert_packages(&self, packages: &[CachedPackage]) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let tx = conn.unchecked_transaction()
            .context("Failed to start transaction")?;

        for package in packages {
            tx.execute(
                "INSERT OR REPLACE INTO packages
                 (name, version, description, attr_path, last_updated, search_count)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                    COALESCE((SELECT search_count FROM packages WHERE attr_path = ?4), 0)
                 )",
                params![
                    package.name,
                    package.version,
                    package.description,
                    package.attr_path,
                    package.last_updated,
                ],
            ).context("Failed to insert package")?;
        }

        tx.commit().context("Failed to commit transaction")?;

        info!("Upserted {} packages to cache", packages.len());
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let total_packages: i32 = conn.query_row(
            "SELECT COUNT(*) FROM packages",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let last_updated: Option<i64> = conn.query_row(
            "SELECT MAX(last_updated) FROM packages",
            [],
            |row| row.get(0)
        ).ok();

        Ok(CacheStats {
            total_packages,
            last_updated,
        })
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.stats().map(|s| s.total_packages == 0).unwrap_or(true)
    }

    /// Clear all cached packages
    pub fn clear(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        conn.execute("DELETE FROM packages", [])
            .context("Failed to clear cache")?;

        info!("Package cache cleared");
        Ok(())
    }
}

impl Default for PackageCache {
    fn default() -> Self {
        Self::new().expect("Failed to create package cache")
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_packages: i32,
    pub last_updated: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = PackageCache::new();
        assert!(cache.is_ok());
    }

    #[test]
    fn test_cache_initialization() {
        let cache = PackageCache::new().unwrap();
        assert!(cache.initialize().is_ok());
    }

    #[test]
    fn test_search_empty_cache() {
        let cache = PackageCache::new().unwrap();
        cache.initialize().unwrap();

        let results = cache.search("test", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_upsert_and_search() {
        let cache = PackageCache::new().unwrap();
        cache.initialize().unwrap();
        cache.clear().unwrap();

        let packages = vec![
            CachedPackage {
                name: "hello".to_string(),
                version: "2.12".to_string(),
                description: "A program that produces a familiar greeting".to_string(),
                attr_path: "nixpkgs.hello".to_string(),
                last_updated: chrono::Utc::now().timestamp(),
                search_count: 0,
            },
        ];

        cache.upsert_packages(&packages).unwrap();

        let results = cache.search("hello", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "hello");
    }
}
