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
#[derive(Clone)]
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

        // Installation history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                action TEXT NOT NULL,
                package_name TEXT NOT NULL,
                version TEXT,
                success INTEGER NOT NULL,
                error_message TEXT
            )",
            [],
        ).context("Failed to create history table")?;

        // Create index on timestamp for fast history queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp DESC)",
            [],
        ).context("Failed to create history timestamp index")?;

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

    /// Search with fuzzy matching and relevance scoring
    pub fn fuzzy_search(&self, query: &str, limit: usize) -> Result<Vec<CachedPackage>> {
        use fuzzy_matcher::FuzzyMatcher;
        use fuzzy_matcher::skim::SkimMatcherV2;

        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        // Get all packages (or a broader set for fuzzy matching)
        let mut stmt = conn.prepare(
            "SELECT name, version, description, attr_path, last_updated, search_count
             FROM packages"
        ).context("Failed to prepare query")?;

        let packages = stmt.query_map([], |row| {
            Ok(CachedPackage {
                name: row.get(0)?,
                version: row.get(1)?,
                description: row.get(2)?,
                attr_path: row.get(3)?,
                last_updated: row.get(4)?,
                search_count: row.get(5)?,
            })
        }).context("Failed to execute query")?
          .collect::<SqlResult<Vec<_>>>()
          .context("Failed to collect results")?;

        // Use fuzzy matcher for scoring
        let matcher = SkimMatcherV2::default();
        let mut scored_packages: Vec<(CachedPackage, i64)> = packages
            .into_iter()
            .filter_map(|pkg| {
                // Score against package name (higher weight)
                let name_score = matcher.fuzzy_match(&pkg.name, query).map(|s| s * 2);

                // Score against description (lower weight)
                let desc_score = matcher.fuzzy_match(&pkg.description, query);

                // Combine scores, prefer name matches
                let total_score = name_score.or(desc_score)?;

                // Boost score slightly for popular packages
                let popularity_bonus = (pkg.search_count as i64).min(50);
                let final_score = total_score + popularity_bonus;

                Some((pkg, final_score))
            })
            .collect();

        // Sort by score (descending)
        scored_packages.sort_by(|a, b| b.1.cmp(&a.1));

        // Take top results
        let results: Vec<CachedPackage> = scored_packages
            .into_iter()
            .take(limit)
            .map(|(pkg, _score)| pkg)
            .collect();

        debug!("Fuzzy search for '{}': found {} results", query, results.len());

        // Increment search counts for found packages
        if !results.is_empty() {
            self.increment_search_counts(&results)?;
        }

        Ok(results)
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

        let packages_with_searches: i32 = conn.query_row(
            "SELECT COUNT(*) FROM packages WHERE search_count > 0",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let total_searches: i32 = conn.query_row(
            "SELECT COALESCE(SUM(search_count), 0) FROM packages",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let average_description_length: f64 = conn.query_row(
            "SELECT COALESCE(AVG(LENGTH(description)), 0.0) FROM packages",
            [],
            |row| row.get(0)
        ).unwrap_or(0.0);

        let oldest_package: Option<i64> = conn.query_row(
            "SELECT MIN(last_updated) FROM packages",
            [],
            |row| row.get(0)
        ).ok();

        let newest_package: Option<i64> = conn.query_row(
            "SELECT MAX(last_updated) FROM packages",
            [],
            |row| row.get(0)
        ).ok();

        Ok(CacheStats {
            total_packages,
            last_updated,
            packages_with_searches,
            total_searches,
            average_description_length,
            oldest_package,
            newest_package,
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

        conn.execute("DELETE FROM metadata", [])
            .context("Failed to clear metadata")?;

        info!("Package cache cleared");
        Ok(())
    }

    /// Get cache database file size in bytes
    pub fn get_size(&self) -> Result<u64> {
        let metadata = std::fs::metadata(&self.db_path)
            .context("Failed to get database file metadata")?;
        Ok(metadata.len())
    }

    /// Get cache database path
    pub fn get_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Get most popular packages (by search count)
    pub fn get_popular(&self, limit: usize) -> Result<Vec<CachedPackage>> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let mut stmt = conn.prepare(
            "SELECT name, version, description, attr_path, last_updated, search_count
             FROM packages
             WHERE search_count > 0
             ORDER BY search_count DESC
             LIMIT ?1"
        ).context("Failed to prepare popular packages query")?;

        let packages = stmt.query_map(params![limit as i32], |row| {
            Ok(CachedPackage {
                name: row.get(0)?,
                version: row.get(1)?,
                description: row.get(2)?,
                attr_path: row.get(3)?,
                last_updated: row.get(4)?,
                search_count: row.get(5)?,
            })
        }).context("Failed to execute query")?
          .collect::<SqlResult<Vec<_>>>()
          .context("Failed to collect results")?;

        Ok(packages)
    }

    /// Get cache age in seconds (time since last update)
    pub fn get_age_seconds(&self) -> Result<Option<i64>> {
        let stats = self.stats()?;

        if let Some(last_updated) = stats.last_updated {
            let now = chrono::Utc::now().timestamp();
            Ok(Some(now - last_updated))
        } else {
            Ok(None)
        }
    }

    /// Get cache health status
    pub fn get_health(&self) -> Result<CacheHealth> {
        let stats = self.stats()?;

        if stats.total_packages == 0 {
            return Ok(CacheHealth::Empty);
        }

        if let Some(age_secs) = self.get_age_seconds()? {
            let age_days = age_secs / 86400;

            if age_days < 7 {
                Ok(CacheHealth::Fresh)
            } else if age_days < 30 {
                Ok(CacheHealth::Good)
            } else if age_days < 90 {
                Ok(CacheHealth::Stale)
            } else {
                Ok(CacheHealth::Outdated)
            }
        } else {
            Ok(CacheHealth::Empty)
        }
    }

    /// Get cache effectiveness percentage (0-100)
    /// Represents the percentage of packages that have been searched
    pub fn get_effectiveness(&self) -> Result<f64> {
        let stats = self.stats()?;

        if stats.total_packages == 0 {
            return Ok(0.0);
        }

        let effectiveness = (stats.packages_with_searches as f64 / stats.total_packages as f64) * 100.0;
        Ok(effectiveness)
    }

    /// Record an installation history entry
    pub fn record_history(
        &self,
        action: HistoryAction,
        package_name: &str,
        version: Option<&str>,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let timestamp = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO history (timestamp, action, package_name, version, success, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                timestamp,
                action.as_str(),
                package_name,
                version,
                success as i32,
                error_message,
            ],
        ).context("Failed to record history entry")?;

        debug!("Recorded {} action for package '{}'", action.as_str(), package_name);
        Ok(())
    }

    /// Get recent installation history
    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, package_name, version, success, error_message
             FROM history
             ORDER BY timestamp DESC
             LIMIT ?1"
        ).context("Failed to prepare history query")?;

        let entries = stmt.query_map(params![limit as i32], |row| {
            let action_str: String = row.get(2)?;
            let action = action_str.parse()
                .unwrap_or(HistoryAction::Install);

            Ok(HistoryEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action,
                package_name: row.get(3)?,
                version: row.get(4)?,
                success: row.get::<_, i32>(5)? != 0,
                error_message: row.get(6)?,
            })
        }).context("Failed to execute history query")?
          .collect::<SqlResult<Vec<_>>>()
          .context("Failed to collect history entries")?;

        Ok(entries)
    }

    /// Get installation history for a specific package
    pub fn get_package_history(&self, package_name: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, package_name, version, success, error_message
             FROM history
             WHERE package_name = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        ).context("Failed to prepare package history query")?;

        let entries = stmt.query_map(params![package_name, limit as i32], |row| {
            let action_str: String = row.get(2)?;
            let action = action_str.parse()
                .unwrap_or(HistoryAction::Install);

            Ok(HistoryEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action,
                package_name: row.get(3)?,
                version: row.get(4)?,
                success: row.get::<_, i32>(5)? != 0,
                error_message: row.get(6)?,
            })
        }).context("Failed to execute package history query")?
          .collect::<SqlResult<Vec<_>>>()
          .context("Failed to collect history entries")?;

        Ok(entries)
    }

    /// Get history statistics
    pub fn get_history_stats(&self) -> Result<HistoryStats> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let total_operations: i32 = conn.query_row(
            "SELECT COUNT(*) FROM history",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let successful_operations: i32 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE success = 1",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let failed_operations: i32 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE success = 0",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let total_installs: i32 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE action = 'install' AND success = 1",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let total_removes: i32 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE action = 'remove' AND success = 1",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let last_operation: Option<i64> = conn.query_row(
            "SELECT MAX(timestamp) FROM history",
            [],
            |row| row.get(0)
        ).ok();

        Ok(HistoryStats {
            total_operations,
            successful_operations,
            failed_operations,
            total_installs,
            total_removes,
            last_operation,
        })
    }

    /// Clear installation history
    pub fn clear_history(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open database")?;

        conn.execute("DELETE FROM history", [])
            .context("Failed to clear history")?;

        info!("Installation history cleared");
        Ok(())
    }
}

impl Default for PackageCache {
    fn default() -> Self {
        Self::new().expect("Failed to create package cache")
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_packages: i32,
    pub last_updated: Option<i64>,
    pub packages_with_searches: i32,
    pub total_searches: i32,
    pub average_description_length: f64,
    pub oldest_package: Option<i64>,
    pub newest_package: Option<i64>,
}

/// Cache health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheHealth {
    Empty,
    Fresh,      // < 7 days
    Good,       // 7-30 days
    Stale,      // 30-90 days
    Outdated,   // > 90 days
}

/// Installation action type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryAction {
    Install,
    Remove,
    Upgrade,
    Update,
}

impl HistoryAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            HistoryAction::Install => "install",
            HistoryAction::Remove => "remove",
            HistoryAction::Upgrade => "upgrade",
            HistoryAction::Update => "update",
        }
    }
}

impl std::str::FromStr for HistoryAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "install" => Ok(HistoryAction::Install),
            "remove" => Ok(HistoryAction::Remove),
            "upgrade" => Ok(HistoryAction::Upgrade),
            "update" => Ok(HistoryAction::Update),
            _ => Err(format!("Invalid history action: {}", s)),
        }
    }
}

/// Installation history entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: i64,
    pub action: HistoryAction,
    pub package_name: String,
    pub version: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Installation history statistics
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_operations: i32,
    pub successful_operations: i32,
    pub failed_operations: i32,
    pub total_installs: i32,
    pub total_removes: i32,
    pub last_operation: Option<i64>,
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
