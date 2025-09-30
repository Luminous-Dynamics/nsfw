use thiserror::Error;

#[derive(Error, Debug)]
pub enum NixError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Nix command failed: {0}")]
    CommandFailed(String),

    #[error("JSON parsing error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Package already installed: {0}")]
    AlreadyInstalled(String),

    #[error("Package not installed: {0}")]
    NotInstalled(String),

    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),

    #[error("Nix not found. Please install Nix: https://nixos.org/download.html")]
    NixNotInstalled,
}

pub type Result<T> = std::result::Result<T, NixError>;