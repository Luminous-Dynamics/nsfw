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

    #[error("Operation timeout after {0} seconds")]
    Timeout(u64),

    #[error("Disk space insufficient: {0}")]
    DiskSpaceLow(String),

    #[error("Package conflict: {0}")]
    PackageConflict(String),

    #[error("WSL2 distribution not found: {0}")]
    WSL2DistroNotFound(String),

    #[error("Nix channel update required")]
    ChannelUpdateRequired,
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
            NixError::Timeout(seconds) => {
                format!("Operation timed out after {} seconds", seconds)
            }
            NixError::DiskSpaceLow(msg) => {
                format!("Insufficient disk space: {}", msg)
            }
            NixError::PackageConflict(msg) => {
                format!("Package conflict detected: {}", msg)
            }
            NixError::WSL2DistroNotFound(distro) => {
                format!("WSL2 distribution '{}' not found", distro)
            }
            NixError::ChannelUpdateRequired => {
                "Your Nix channels need to be updated".to_string()
            }
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            NixError::PackageNotFound(pkg) => {
                Some(format!(
                    "Troubleshooting steps:\n\
                     1. Search with fuzzy matching: nsfw search {}\n\
                     2. Try alternative names (e.g., 'python' → 'python3', 'node' → 'nodejs')\n\
                     3. Update package database: nsfw update\n\
                     4. Search online: https://search.nixos.org/packages?query={}",
                    pkg, pkg
                ))
            }
            NixError::NetworkError(msg) => {
                let suggestion = if msg.contains("timeout") || msg.contains("timed out") {
                    "Network timeout detected:\n\
                     1. Check your internet connection: ping google.com\n\
                     2. Verify WSL2 has network access: wsl ping google.com\n\
                     3. Check firewall and antivirus settings\n\
                     4. Try again with slower network"
                } else if msg.contains("connection refused") || msg.contains("refused") {
                    "Connection refused:\n\
                     1. Check if WSL2 is running: wsl -l -v\n\
                     2. Restart WSL2: wsl --shutdown && wsl\n\
                     3. Verify network configuration\n\
                     4. Check if a VPN is interfering"
                } else {
                    "Network error resolution:\n\
                     1. Test internet: ping google.com\n\
                     2. Check firewall settings\n\
                     3. Disable VPN temporarily\n\
                     4. Try again in a few moments"
                };
                Some(suggestion.to_string())
            }
            NixError::AlreadyInstalled(pkg) => {
                Some(format!(
                    "Package '{}' is already available.\n\n\
                     Options:\n\
                     • Skip (package is ready to use)\n\
                     • Upgrade: nsfw upgrade {}\n\
                     • Reinstall:\n\
                       1. nsfw remove {}\n\
                       2. nsfw install {}",
                    pkg, pkg, pkg, pkg
                ))
            }
            NixError::NotInstalled(pkg) => {
                Some(format!(
                    "Package '{}' is not installed.\n\n\
                     Next steps:\n\
                     1. List installed packages: nsfw list\n\
                     2. Search for it: nsfw search {}\n\
                     3. Install if found: nsfw install {}",
                    pkg, pkg, pkg
                ))
            }
            NixError::NixNotInstalled => {
                Some(
                    "Nix installation required:\n\n\
                     EASY METHOD (Recommended):\n\
                     → Run: nsfw setup\n\n\
                     MANUAL METHOD:\n\
                     1. Open WSL2: wsl\n\
                     2. Install Nix (multi-user):\n\
                        curl -L https://nixos.org/nix/install | sh -s -- --daemon\n\
                     3. Restart WSL2\n\
                     4. Verify: nix --version"
                        .to_string(),
                )
            }
            NixError::WSL2NotAvailable => {
                Some(
                    "WSL2 installation required:\n\n\
                     EASY METHOD (Recommended):\n\
                     → Run: nsfw setup\n\n\
                     MANUAL METHOD:\n\
                     1. Open PowerShell as Administrator\n\
                     2. Run: wsl --install\n\
                     3. Restart your computer\n\
                     4. Complete Ubuntu setup\n\
                     5. Run: nsfw setup\n\n\
                     TROUBLESHOOTING:\n\
                     • Ensure virtualization is enabled in BIOS\n\
                     • Windows 10 2004+ or Windows 11 required\n\
                     • Check: wsl --status"
                        .to_string(),
                )
            }
            NixError::CacheError(msg) => {
                let suggestion = if msg.contains("corrupt") || msg.contains("invalid") {
                    "Cache corruption detected:\n\
                     1. Clear cache: nsfw cache clear\n\
                     2. Rebuild: nsfw cache rebuild\n\
                     3. Update channels: nsfw update"
                } else {
                    "Cache error resolution:\n\
                     1. Check disk space: disk usage\n\
                     2. Clear cache: nsfw cache clear\n\
                     3. Rebuild cache: nsfw cache rebuild\n\
                     4. Check permissions"
                };
                Some(suggestion.to_string())
            }
            NixError::PermissionDenied(msg) => {
                let suggestion = if msg.contains("WSL") || msg.contains("wsl") {
                    "WSL2 permission issue:\n\
                     1. Restart WSL2: wsl --shutdown && wsl\n\
                     2. Check file ownership in WSL2\n\
                     3. Run: nsfw doctor (for diagnostics)"
                } else {
                    "Permission denied - Quick fixes:\n\
                     1. Run PowerShell as Administrator\n\
                     2. Check file/folder permissions\n\
                     3. Ensure antivirus isn't blocking\n\
                     4. Verify WSL2 access: wsl whoami"
                };
                Some(suggestion.to_string())
            }
            NixError::Timeout(seconds) => {
                Some(format!(
                    "Operation timed out after {} seconds.\n\n\
                     Possible causes:\n\
                     • Slow internet connection\n\
                     • Large package download\n\
                     • WSL2 performance issues\n\n\
                     Solutions:\n\
                     1. Check network speed\n\
                     2. Try again (may work on retry)\n\
                     3. Restart WSL2: wsl --shutdown\n\
                     4. Close other applications",
                    seconds
                ))
            }
            NixError::DiskSpaceLow(details) => {
                Some(format!(
                    "Insufficient disk space: {}\n\n\
                     Free up space:\n\
                     1. Clear NSFW cache: nsfw cache clear\n\
                     2. Remove unused packages: nsfw list (then remove)\n\
                     3. Clean WSL2: wsl nix-collect-garbage -d\n\
                     4. Free up Windows disk space\n\
                     5. Check space: wsl df -h",
                    details
                ))
            }
            NixError::PackageConflict(details) => {
                Some(format!(
                    "Package conflict: {}\n\n\
                     Resolution options:\n\
                     1. Remove conflicting package first\n\
                     2. Try upgrading instead: nsfw upgrade\n\
                     3. Use --force flag (if available)\n\
                     4. Check for dependency issues: nsfw info <package>",
                    details
                ))
            }
            NixError::WSL2DistroNotFound(distro) => {
                Some(format!(
                    "WSL2 distribution '{}' not found.\n\n\
                     Available options:\n\
                     1. List distributions: wsl -l -v\n\
                     2. Install Ubuntu: wsl --install -d Ubuntu\n\
                     3. Set default: wsl --set-default Ubuntu\n\
                     4. Run setup: nsfw setup",
                    distro
                ))
            }
            NixError::ChannelUpdateRequired => {
                Some(
                    "Your Nix channels are outdated.\n\n\
                     Update channels:\n\
                     → nsfw update\n\n\
                     This ensures you get:\n\
                     • Latest package versions\n\
                     • Security updates\n\
                     • Bug fixes\n\
                     • New packages"
                        .to_string(),
                )
            }
            NixError::InvalidPackageName(_) => {
                Some(
                    "Invalid package name format.\n\n\
                     Package names should:\n\
                     • Contain only letters, numbers, hyphens, underscores\n\
                     • Not start with a number\n\
                     • Be lowercase (usually)\n\n\
                     Examples: firefox, python3, nodejs-20"
                        .to_string(),
                )
            }
            _ => None,
        }
    }

    fn help_url(&self) -> Option<String> {
        match self {
            NixError::NixNotInstalled => {
                Some("https://nixos.org/download.html".to_string())
            }
            NixError::WSL2NotAvailable | NixError::WSL2DistroNotFound(_) => {
                Some("https://docs.microsoft.com/en-us/windows/wsl/install".to_string())
            }
            NixError::PackageNotFound(pkg) => {
                Some(format!("https://search.nixos.org/packages?query={}", pkg))
            }
            NixError::NetworkError(_) | NixError::Timeout(_) => {
                Some("https://nixos.org/manual/nix/stable/#sec-network-troubleshooting".to_string())
            }
            NixError::ChannelUpdateRequired => {
                Some("https://nixos.org/manual/nix/stable/#sec-channels".to_string())
            }
            NixError::DiskSpaceLow(_) => {
                Some("https://nixos.org/manual/nix/stable/#sec-garbage-collection".to_string())
            }
            NixError::PermissionDenied(_) => {
                Some("https://docs.microsoft.com/en-us/windows/wsl/troubleshooting".to_string())
            }
            _ => None,
        }
    }
}