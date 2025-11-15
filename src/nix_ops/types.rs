use serde::{Deserialize, Serialize};

/// A package in nixpkgs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package attribute path (e.g., "nixpkgs#firefox")
    pub pname: String,

    /// Package version
    pub version: String,

    /// Short description
    pub description: String,
}

/// Search result from nix search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Package name
    pub pname: String,

    /// Package version
    pub version: String,

    /// Description
    pub description: String,
}

/// Installed package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// Package name
    pub name: String,

    /// Package version
    pub version: String,

    /// Store path
    pub store_path: String,
}

/// Detailed package information (from nix search or flake show)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package attribute name
    pub pname: String,

    /// Package version
    pub version: String,

    /// Full description
    pub description: String,

    /// Homepage URL (if available)
    pub homepage: Option<String>,

    /// License (if available)
    pub license: Option<String>,

    /// Available outputs (bin, dev, doc, etc.)
    pub outputs: Vec<String>,

    /// Maintainers (if available)
    pub maintainers: Vec<String>,

    /// Platforms supported
    pub platforms: Vec<String>,
}