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

    #[error("WSL2 is not available. Please install WSL2: https://docs.microsoft.com/en-us/windows/wsl/install")]
    WSL2NotAvailable,

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, NixError>;

/// Extension trait for enhanced error messages with context and suggestions
pub trait ErrorContext {
    /// Get a user-friendly error message with suggestions
    fn user_message(&self) -> String;

    /// Get actionable suggestion for fixing the error
    fn suggestion(&self) -> Option<String>;

    /// Get help URL if available
    fn help_url(&self) -> Option<String>;
}

impl ErrorContext for NixError {
    fn user_message(&self) -> String {
        match self {
            NixError::PackageNotFound(pkg) => {
                format!("Package '{}' not found in nixpkgs", pkg)
            }
            NixError::NetworkError(msg) => {
                format!("Network connection failed: {}", msg)
            }
            NixError::CommandFailed(msg) => {
                format!("Command execution failed: {}", msg)
            }
            NixError::ParseError(err) => {
                format!("Failed to parse response: {}", err)
            }
            NixError::IoError(err) => {
                format!("File system error: {}", err)
            }
            NixError::AlreadyInstalled(pkg) => {
                format!("Package '{}' is already installed", pkg)
            }
            NixError::NotInstalled(pkg) => {
                format!("Package '{}' is not currently installed", pkg)
            }
            NixError::InvalidPackageName(pkg) => {
                format!("Invalid package name: '{}'", pkg)
            }
            NixError::NixNotInstalled => {
                "Nix package manager is not installed".to_string()
            }
            NixError::WSL2NotAvailable => {
                "WSL2 is not installed or not running".to_string()
            }
            NixError::CacheError(msg) => {
                format!("Package cache error: {}", msg)
            }
            NixError::PermissionDenied(msg) => {
                format!("Permission denied: {}", msg)
            }
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            NixError::PackageNotFound(_) => {
                Some("Try:\n  • Search with a broader term: nsfw search <keyword>\n  • Check spelling and try alternative names\n  • Update package database: nsfw update".to_string())
            }
            NixError::NetworkError(_) => {
                Some("Try:\n  • Check your internet connection\n  • Verify firewall settings\n  • Try again in a few moments".to_string())
            }
            NixError::AlreadyInstalled(pkg) => {
                Some(format!("Package is already available. To reinstall:\n  1. Remove it: nsfw remove {}\n  2. Install again: nsfw install {}", pkg, pkg))
            }
            NixError::NotInstalled(_) => {
                Some("Try:\n  • List installed packages: nsfw list\n  • Search for the package: nsfw search <name>\n  • Check package spelling".to_string())
            }
            NixError::NixNotInstalled => {
                Some("Run the setup wizard:\n  nsfw setup\n\nOr install manually:\n  wsl\n  curl -L https://nixos.org/nix/install | sh -s -- --daemon".to_string())
            }
            NixError::WSL2NotAvailable => {
                Some("Install WSL2:\n  1. Run in PowerShell (as Admin): wsl --install\n  2. Restart your computer\n  3. Run: nsfw setup\n\nOr use the setup wizard: nsfw setup".to_string())
            }
            NixError::CacheError(_) => {
                Some("Try:\n  • Clear cache: nsfw cache clear\n  • Rebuild cache: nsfw cache rebuild\n  • Check disk space".to_string())
            }
            NixError::PermissionDenied(_) => {
                Some("Try:\n  • Run PowerShell as Administrator\n  • Check file permissions\n  • Ensure WSL2 is properly configured".to_string())
            }
            _ => None,
        }
    }

    fn help_url(&self) -> Option<String> {
        match self {
            NixError::NixNotInstalled => {
                Some("https://nixos.org/download.html".to_string())
            }
            NixError::WSL2NotAvailable => {
                Some("https://docs.microsoft.com/en-us/windows/wsl/install".to_string())
            }
            NixError::PackageNotFound(_) => {
                Some("https://search.nixos.org/packages".to_string())
            }
            _ => None,
        }
    }
}