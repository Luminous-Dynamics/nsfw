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