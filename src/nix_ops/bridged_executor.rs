/// Bridged Nix executor that uses WSL2Bridge for all operations
///
/// This executor routes all Nix commands through a WSL2Bridge implementation,
/// enabling Windows CLI to execute Nix operations in WSL2.
use anyhow::Result;
use serde_json::Value;

use super::errors::NixError;
use super::types::{SearchResult, InstalledPackage};
use crate::wsl2::WSL2Bridge;

/// Nix executor that uses WSL2Bridge for all operations
///
/// Generic over the bridge type to allow both real and mock bridges.
pub struct BridgedNixExecutor<B: WSL2Bridge> {
    bridge: B,
}

impl<B: WSL2Bridge> BridgedNixExecutor<B> {
    /// Create a new bridged executor
    ///
    /// # Arguments
    /// * `bridge` - WSL2Bridge implementation to use
    ///
    /// # Example
    /// ```
    /// use nsfw::wsl2::RealWSL2Bridge;
    /// use nsfw::nix_ops::BridgedNixExecutor;
    ///
    /// let bridge = RealWSL2Bridge::new();
    /// let executor = BridgedNixExecutor::new(bridge);
    /// ```
    pub fn new(bridge: B) -> Self {
        Self { bridge }
    }

    /// Check if Nix is available in WSL2
    pub fn check_nix_available(&self) -> Result<String, NixError> {
        // First check if WSL2 is available
        if !self.bridge.is_available() {
            return Err(NixError::WSL2NotAvailable);
        }

        // Try to get Nix version
        match self.bridge.execute("nix", &["--version"]) {
            Ok(output) if output.is_success() => {
                Ok(output.stdout.trim().to_string())
            }
            Ok(output) => {
                Err(NixError::CommandFailed(format!(
                    "Nix not found in WSL2: {}",
                    output.stderr
                )))
            }
            Err(e) => {
                Err(NixError::CommandFailed(format!(
                    "Failed to check Nix: {}",
                    e
                )))
            }
        }
    }

    /// Search for packages
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, NixError> {
        // Check WSL2 available
        if !self.bridge.is_available() {
            return Err(NixError::WSL2NotAvailable);
        }

        // Execute search via bridge
        let output = self.bridge
            .execute("nix", &["search", "nixpkgs", query, "--json"])
            .map_err(|e| NixError::CommandFailed(e.to_string()))?;

        if !output.is_success() {
            return Err(NixError::CommandFailed(output.stderr));
        }

        // Parse JSON results
        self.parse_search_json(&output.stdout, limit)
    }

    /// Install a package
    pub fn install(&self, package: &str) -> Result<(), NixError> {
        // Check WSL2 available
        if !self.bridge.is_available() {
            return Err(NixError::WSL2NotAvailable);
        }

        let pkg_attr = format!("nixpkgs#{}", package);

        // Execute install via bridge
        let output = self.bridge
            .execute("nix", &["profile", "install", &pkg_attr])
            .map_err(|e| NixError::CommandFailed(e.to_string()))?;

        if !output.is_success() {
            // Check if already installed
            if output.stderr.contains("already installed") {
                return Err(NixError::AlreadyInstalled(package.to_string()));
            }
            return Err(NixError::CommandFailed(output.stderr));
        }

        Ok(())
    }

    /// Remove a package
    pub fn remove(&self, package: &str) -> Result<(), NixError> {
        // Check WSL2 available
        if !self.bridge.is_available() {
            return Err(NixError::WSL2NotAvailable);
        }

        // Execute remove via bridge
        let output = self.bridge
            .execute("nix", &["profile", "remove", package])
            .map_err(|e| NixError::CommandFailed(e.to_string()))?;

        if !output.is_success() {
            // Check if not installed
            if output.stderr.contains("not found") || output.stderr.contains("does not exist") {
                return Err(NixError::NotInstalled(package.to_string()));
            }
            return Err(NixError::CommandFailed(output.stderr));
        }

        Ok(())
    }

    /// List installed packages
    pub fn list(&self) -> Result<Vec<InstalledPackage>, NixError> {
        // Check WSL2 available
        if !self.bridge.is_available() {
            return Err(NixError::WSL2NotAvailable);
        }

        // Execute list via bridge
        let output = self.bridge
            .execute("nix", &["profile", "list", "--json"])
            .map_err(|e| NixError::CommandFailed(e.to_string()))?;

        if !output.is_success() {
            return Err(NixError::CommandFailed(output.stderr));
        }

        // Parse JSON results
        self.parse_list_json(&output.stdout)
    }

    /// Parse search results from JSON
    fn parse_search_json(&self, json_str: &str, limit: usize) -> Result<Vec<SearchResult>, NixError> {
        let json: Value = serde_json::from_str(json_str)?;

        let mut results = Vec::new();

        if let Some(obj) = json.as_object() {
            for (attr, pkg_info) in obj.iter().take(limit) {
                let pname = pkg_info
                    .get("pname")
                    .and_then(|v| v.as_str())
                    .unwrap_or(attr)
                    .to_string();

                let version = pkg_info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let description = pkg_info
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                results.push(SearchResult {
                    pname,
                    version,
                    description,
                });
            }
        }

        Ok(results)
    }

    /// Parse installed packages from JSON
    fn parse_list_json(&self, json_str: &str) -> Result<Vec<InstalledPackage>, NixError> {
        let json: Value = serde_json::from_str(json_str)?;

        let mut packages = Vec::new();

        if let Some(elements) = json.get("elements").and_then(|v| v.as_object()) {
            for (name, element) in elements {
                let store_path = element
                    .get("storePaths")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

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

    /// Extract version from store path
    fn extract_version(&self, store_path: &str, name: &str) -> String {
        // Store path format: /nix/store/hash-name-version/...
        let parts: Vec<&str> = store_path.split('/').collect();

        if parts.len() >= 4 {
            let package_part = parts[3]; // hash-name-version

            // Try to extract version after name
            if let Some(pos) = package_part.find(&format!("{}-", name)) {
                let after_name = &package_part[pos + name.len() + 1..];

                // Take until next dash (which would be architecture or hash continuation)
                if let Some(dash_pos) = after_name.find('-') {
                    return after_name[..dash_pos].to_string();
                } else {
                    return after_name.to_string();
                }
            }
        }

        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wsl2::MockWSL2Bridge;

    fn create_mock_executor() -> BridgedNixExecutor<MockWSL2Bridge> {
        let mut bridge = MockWSL2Bridge::new();
        bridge.add_common_responses();
        BridgedNixExecutor::new(bridge)
    }

    #[test]
    fn test_bridged_executor_new() {
        let bridge = MockWSL2Bridge::new();
        let _executor = BridgedNixExecutor::new(bridge);
        // Just verify construction
        assert!(true);
    }

    #[test]
    fn test_check_nix_available_success() {
        let executor = create_mock_executor();
        let version = executor.check_nix_available().unwrap();
        assert!(version.contains("nix"));
    }

    #[test]
    fn test_check_nix_available_wsl2_unavailable() {
        let mut bridge = MockWSL2Bridge::new();
        bridge.set_available(false);
        let executor = BridgedNixExecutor::new(bridge);

        let result = executor.check_nix_available();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));
    }

    #[test]
    fn test_search_success() {
        let executor = create_mock_executor();

        // Search should work (returns empty by default from common responses)
        let results = executor.search("test", 10).unwrap();
        assert_eq!(results.len(), 0); // Empty search result from mock
    }

    #[test]
    fn test_search_wsl2_unavailable() {
        let mut bridge = MockWSL2Bridge::new();
        bridge.set_available(false);
        let executor = BridgedNixExecutor::new(bridge);

        let result = executor.search("test", 10);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));
    }

    #[test]
    fn test_list_success() {
        let executor = create_mock_executor();

        // List should work (returns empty by default)
        let packages = executor.list().unwrap();
        assert_eq!(packages.len(), 0); // Empty list from mock
    }

    #[test]
    fn test_list_wsl2_unavailable() {
        let mut bridge = MockWSL2Bridge::new();
        bridge.set_available(false);
        let executor = BridgedNixExecutor::new(bridge);

        let result = executor.list();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));
    }

    #[test]
    fn test_extract_version() {
        let bridge = MockWSL2Bridge::new();
        let executor = BridgedNixExecutor::new(bridge);

        // Test version extraction
        let version = executor.extract_version(
            "/nix/store/abc123-firefox-130.0/bin/firefox",
            "firefox"
        );
        assert_eq!(version, "130.0");

        // Test with no version
        let version = executor.extract_version(
            "/nix/store/abc123-vim",
            "vim"
        );
        assert_eq!(version, "unknown");
    }

    #[test]
    fn test_install_wsl2_unavailable() {
        let mut bridge = MockWSL2Bridge::new();
        bridge.set_available(false);
        let executor = BridgedNixExecutor::new(bridge);

        let result = executor.install("firefox");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));
    }

    #[test]
    fn test_remove_wsl2_unavailable() {
        let mut bridge = MockWSL2Bridge::new();
        bridge.set_available(false);
        let executor = BridgedNixExecutor::new(bridge);

        let result = executor.remove("firefox");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));
    }
}
