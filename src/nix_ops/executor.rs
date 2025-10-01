use std::collections::HashMap;
use std::process::Command;
use log::{debug, info, warn};
use serde_json::Value;

use crate::nix_ops::errors::{NixError, Result};
use crate::nix_ops::types::{SearchResult, InstalledPackage};

/// Executes Nix operations (search, install, remove, list)
pub struct NixExecutor {
    /// Whether to use --json flag for commands
    use_json: bool,
}

impl NixExecutor {
    /// Create a new NixExecutor
    pub fn new() -> Self {
        Self {
            use_json: true,
        }
    }

    /// Check if Nix is installed and accessible
    pub fn check_nix_available(&self) -> Result<String> {
        debug!("Checking if Nix is available...");

        let output = Command::new("which")
            .arg("nix")
            .output()
            .map_err(|_| NixError::NixNotInstalled)?;

        if !output.status.success() {
            return Err(NixError::NixNotInstalled);
        }

        // Get Nix version
        let version_output = Command::new("nix")
            .arg("--version")
            .output()
            .map_err(|e| NixError::IoError(e))?;

        let version = String::from_utf8_lossy(&version_output.stdout)
            .trim()
            .to_string();

        info!("Nix found: {}", version);
        Ok(version)
    }

    /// Search for packages in nixpkgs
    ///
    /// # Arguments
    /// * `query` - Search query (package name or description)
    /// * `limit` - Maximum number of results to return
    ///
    /// # Example
    /// ```
    /// let executor = NixExecutor::new();
    /// let results = executor.search("firefox", 10)?;
    /// ```
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        info!("Searching for '{}' (limit: {})", query, limit);

        // Build command: nix search nixpkgs <query> --json
        let mut cmd = Command::new("nix");
        cmd.arg("--extra-experimental-features")
            .arg("nix-command flakes")
            .arg("search")
            .arg("nixpkgs")
            .arg(query);

        if self.use_json {
            cmd.arg("--json");
        }

        debug!("Executing: {:?}", cmd);

        // Execute command
        let output = cmd.output()
            .map_err(|e| NixError::IoError(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Search failed: {}", stderr);

            if stderr.contains("no results") || stderr.contains("not found") {
                return Ok(Vec::new()); // No results, not an error
            }

            return Err(NixError::CommandFailed(stderr.to_string()));
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Raw output length: {} bytes", stdout.len());

        if stdout.is_empty() {
            return Ok(Vec::new());
        }

        let json: HashMap<String, Value> = serde_json::from_str(&stdout)
            .map_err(|e| NixError::ParseError(e))?;

        // Convert to SearchResult objects
        let mut results = Vec::new();
        for (attr_path, package_info) in json.iter() {
            if results.len() >= limit {
                break;
            }

            // Extract pname, version, description
            let pname = package_info.get("pname")
                .and_then(|v| v.as_str())
                .unwrap_or(attr_path)
                .to_string();

            let version = package_info.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let description = package_info.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(SearchResult {
                pname,
                version,
                description,
            });
        }

        info!("Found {} results", results.len());
        Ok(results)
    }

    /// List installed packages
    ///
    /// # Example
    /// ```
    /// let executor = NixExecutor::new();
    /// let packages = executor.list()?;
    /// ```
    pub fn list(&self) -> Result<Vec<InstalledPackage>> {
        info!("Listing installed packages");

        // Build command: nix profile list --json
        let mut cmd = Command::new("nix");
        cmd.arg("--extra-experimental-features")
            .arg("nix-command flakes")
            .arg("profile")
            .arg("list");

        if self.use_json {
            cmd.arg("--json");
        }

        debug!("Executing: {:?}", cmd);

        // Execute command
        let output = cmd.output()
            .map_err(|e| NixError::IoError(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("List failed: {}", stderr);
            return Err(NixError::CommandFailed(stderr.to_string()));
        }

        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);

        if self.use_json {
            // Parse JSON format
            self.parse_list_json(&stdout)
        } else {
            // Parse text format (fallback)
            self.parse_list_text(&stdout)
        }
    }

    /// Parse JSON output from nix profile list --json
    fn parse_list_json(&self, json_str: &str) -> Result<Vec<InstalledPackage>> {
        if json_str.trim().is_empty() {
            return Ok(Vec::new());
        }

        let json: Value = serde_json::from_str(json_str)
            .map_err(|e| NixError::ParseError(e))?;

        let mut packages = Vec::new();

        // nix profile list --json returns an object with elements object (not array!)
        // Format: { "elements": { "package-name": {...}, ... }, "version": 3 }
        if let Some(elements) = json.get("elements").and_then(|v| v.as_object()) {
            for (name, element) in elements {
                let store_path = element.get("storePaths")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Try to extract version from store path or element
                let version = self.extract_version(&store_path, name);

                packages.push(InstalledPackage {
                    name: name.clone(),
                    version,
                    store_path,
                });
            }
        }

        Ok(packages)
    }

    /// Parse text output from nix profile list (fallback)
    fn parse_list_text(&self, text: &str) -> Result<Vec<InstalledPackage>> {
        let mut packages = Vec::new();

        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Format: <index> <name> <store-path>
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].to_string();
                let store_path = parts.get(2)
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                let version = self.extract_version(&store_path, &name);

                packages.push(InstalledPackage {
                    name,
                    version,
                    store_path,
                });
            }
        }

        Ok(packages)
    }

    /// Extract version from store path or name
    /// Example: /nix/store/abc-firefox-130.0 → "130.0"
    fn extract_version(&self, store_path: &str, name: &str) -> String {
        // Try to extract from store path
        if let Some(pos) = store_path.rfind('-') {
            let potential_version = &store_path[pos + 1..];
            // Check if it looks like a version (starts with digit)
            if potential_version.chars().next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return potential_version.to_string();
            }
        }

        // Try to extract from name
        if let Some(pos) = name.rfind('-') {
            let potential_version = &name[pos + 1..];
            if potential_version.chars().next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return potential_version.to_string();
            }
        }

        "unknown".to_string()
    }

    /// Install a package
    ///
    /// # Arguments
    /// * `package` - Package name (e.g., "firefox", "python3")
    ///
    /// # Example
    /// ```
    /// let executor = NixExecutor::new();
    /// executor.install("firefox")?;
    /// ```
    pub fn install(&self, package: &str) -> Result<()> {
        info!("Installing package: {}", package);

        // Build command: nix profile install nixpkgs#<package>
        let package_ref = if package.contains('#') {
            package.to_string()
        } else {
            format!("nixpkgs#{}", package)
        };

        let mut cmd = Command::new("nix");
        cmd.arg("--extra-experimental-features")
            .arg("nix-command flakes")
            .arg("profile")
            .arg("install")
            .arg(&package_ref);

        debug!("Executing: {:?}", cmd);

        // Execute command
        let output = cmd.output()
            .map_err(|e| NixError::IoError(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Install failed: {}", stderr);

            // Check for specific error conditions
            if stderr.contains("already installed") {
                return Err(NixError::AlreadyInstalled(package.to_string()));
            }
            if stderr.contains("not found") || stderr.contains("does not exist") {
                return Err(NixError::PackageNotFound(package.to_string()));
            }
            if stderr.contains("network") || stderr.contains("download") {
                return Err(NixError::NetworkError(stderr.to_string()));
            }

            return Err(NixError::CommandFailed(stderr.to_string()));
        }

        info!("Successfully installed: {}", package);
        Ok(())
    }

    /// Remove a package
    ///
    /// # Arguments
    /// * `package` - Package name to remove
    ///
    /// # Example
    /// ```
    /// let executor = NixExecutor::new();
    /// executor.remove("firefox")?;
    /// ```
    pub fn remove(&self, package: &str) -> Result<()> {
        info!("Removing package: {}", package);

        // Build command: nix profile remove <package>
        let mut cmd = Command::new("nix");
        cmd.arg("--extra-experimental-features")
            .arg("nix-command flakes")
            .arg("profile")
            .arg("remove")
            .arg(package);

        debug!("Executing: {:?}", cmd);

        // Execute command
        let output = cmd.output()
            .map_err(|e| NixError::IoError(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Remove failed: {}", stderr);

            if stderr.contains("not found") || stderr.contains("not installed") {
                return Err(NixError::NotInstalled(package.to_string()));
            }

            return Err(NixError::CommandFailed(stderr.to_string()));
        }

        info!("Successfully removed: {}", package);
        Ok(())
    }
}

impl Default for NixExecutor {
    fn default() -> Self {
        Self::new()
    }
}