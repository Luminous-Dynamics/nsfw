/// Background cache builder
///
/// This module handles the async building of the package cache
/// without blocking the main thread or user operations.
use super::{PackageCache, CachedPackage};
use crate::wsl2::WSL2Bridge;
use anyhow::{Result, Context};
use log::{info, debug};
use serde_json::Value;
use std::collections::HashMap;

/// Background cache builder
pub struct CacheBuilder<B: WSL2Bridge> {
    cache: PackageCache,
    bridge: B,
}

impl<B: WSL2Bridge> CacheBuilder<B> {
    /// Create a new cache builder
    pub fn new(cache: PackageCache, bridge: B) -> Self {
        Self { cache, bridge }
    }

    /// Build the cache from nix-env -qa
    ///
    /// This is faster than `nix search` because:
    /// 1. Uses older nix-env command (no flakes needed)
    /// 2. Doesn't require eval-cache download
    /// 3. Can be run in background
    pub fn build_from_nix_env(&self) -> Result<usize> {
        info!("Starting background cache build from nix-env");

        // Use nix-env -qaP --json for fast package list
        let output = self.bridge.execute(
            "nix-env",
            &["-qaP", "--json"]
        ).context("Failed to execute nix-env")?;

        if !output.is_success() {
            return Err(anyhow::anyhow!("nix-env failed: {}", output.stderr));
        }

        // Parse JSON output
        let packages_json: HashMap<String, Value> = serde_json::from_str(&output.stdout)
            .context("Failed to parse nix-env JSON output")?;

        // Convert to CachedPackage objects
        let mut packages = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for (attr_path, package_info) in packages_json.iter() {
            // Extract package information
            let name = package_info.get("pname")
                .or_else(|| package_info.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or(attr_path)
                .to_string();

            let version = package_info.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let description = package_info.get("meta")
                .and_then(|m| m.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();

            packages.push(CachedPackage {
                name,
                version,
                description,
                attr_path: attr_path.clone(),
                last_updated: now,
                search_count: 0,
            });

            // Batch upsert every 1000 packages
            if packages.len() >= 1000 {
                debug!("Upserting batch of 1000 packages");
                self.cache.upsert_packages(&packages)
                    .context("Failed to upsert package batch")?;
                packages.clear();
            }
        }

        // Upsert remaining packages
        if !packages.is_empty() {
            self.cache.upsert_packages(&packages)
                .context("Failed to upsert final package batch")?;
        }

        let stats = self.cache.stats()?;
        info!("Cache build complete: {} packages cached", stats.total_packages);

        Ok(stats.total_packages as usize)
    }

    /// Quick build - just cache popular packages
    ///
    /// This provides instant results for common searches
    /// while the full cache builds in background
    pub fn build_popular_packages(&self) -> Result<usize> {
        info!("Quick-building cache with popular packages");

        let popular_packages = vec![
            // Programming languages
            "python3", "python310", "python311", "python312",
            "nodejs", "nodejs_20", "nodejs_22",
            "go", "rustc", "cargo",
            "ruby", "ruby_3_2", "ruby_3_3",
            "php", "php83",
            "java", "openjdk", "jdk17", "jdk21",

            // Development tools
            "git", "gh", "vim", "neovim", "emacs",
            "vscode", "code", "jetbrains",
            "docker", "docker-compose", "kubectl",
            "terraform", "ansible",

            // Databases
            "postgresql", "mysql", "redis", "mongodb",
            "sqlite",

            // Build tools
            "cmake", "make", "ninja", "meson",
            "gcc", "clang", "llvm",

            // Common utilities
            "curl", "wget", "jq", "ripgrep", "fd",
            "bat", "exa", "htop", "tmux", "zsh",
        ];

        let mut count = 0;
        let now = chrono::Utc::now().timestamp();

        for package_name in popular_packages {
            // Try to get package info from nix-env
            match self.get_package_info(package_name) {
                Ok(Some(mut pkg)) => {
                    pkg.last_updated = now;
                    pkg.search_count = 100; // Give popular packages high initial count
                    self.cache.upsert_packages(&[pkg])?;
                    count += 1;
                }
                Ok(None) => debug!("Popular package '{}' not found", package_name),
                Err(e) => debug!("Failed to get info for '{}': {}", package_name, e),
            }
        }

        info!("Quick-cached {} popular packages", count);
        Ok(count)
    }

    /// Get package info for a single package
    fn get_package_info(&self, package_name: &str) -> Result<Option<CachedPackage>> {
        let output = self.bridge.execute(
            "nix-env",
            &["-qaP", "--json", package_name]
        )?;

        if !output.is_success() {
            return Ok(None);
        }

        let packages_json: HashMap<String, Value> = serde_json::from_str(&output.stdout)
            .context("Failed to parse package info")?;

        // Get the first package (usually the one we want)
        if let Some((attr_path, package_info)) = packages_json.iter().next() {
            let name = package_info.get("pname")
                .or_else(|| package_info.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or(package_name)
                .to_string();

            let version = package_info.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let description = package_info.get("meta")
                .and_then(|m| m.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();

            Ok(Some(CachedPackage {
                name,
                version,
                description,
                attr_path: attr_path.clone(),
                last_updated: chrono::Utc::now().timestamp(),
                search_count: 0,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if cache needs update (>24 hours old)
    pub fn needs_update(&self) -> Result<bool> {
        let stats = self.cache.stats()?;

        if stats.total_packages == 0 {
            return Ok(true); // Empty cache needs building
        }

        if let Some(last_updated) = stats.last_updated {
            let age_hours = (chrono::Utc::now().timestamp() - last_updated) / 3600;
            Ok(age_hours > 24)
        } else {
            Ok(true) // No timestamp means needs update
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wsl2::MockWSL2Bridge;

    #[test]
    fn test_cache_builder_creation() {
        let cache = PackageCache::new().unwrap();
        cache.initialize().unwrap();

        let bridge = MockWSL2Bridge::new();
        let _builder = CacheBuilder::new(cache, bridge);

        // Just verify construction works
        assert!(true);
    }
}
