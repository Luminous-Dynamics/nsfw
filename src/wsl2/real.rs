/// Real WSL2 bridge implementation using wsl.exe
use std::process::Command;
use anyhow::{Result, anyhow};

use super::bridge::{WSL2Bridge, CommandOutput};
use crate::path_translation::PathTranslator;

/// Real WSL2 bridge that executes commands via wsl.exe
pub struct RealWSL2Bridge {
    /// Path translator for Windows ↔ WSL path conversion
    path_translator: PathTranslator,
}

impl RealWSL2Bridge {
    /// Create a new real WSL2 bridge
    pub fn new() -> Self {
        Self {
            path_translator: PathTranslator::new(),
        }
    }

    /// Translate any Windows paths in arguments to WSL paths
    fn translate_args(&self, args: &[&str]) -> Result<Vec<String>> {
        let mut translated = Vec::new();

        for arg in args {
            // Check if this looks like a Windows path (has drive letter)
            if self.is_windows_path(arg) {
                translated.push(self.translate_path_to_wsl(arg)?);
            } else {
                translated.push(arg.to_string());
            }
        }

        Ok(translated)
    }

    /// Check if a string looks like a Windows path
    fn is_windows_path(&self, path: &str) -> bool {
        // Simple heuristic: has drive letter pattern (C:\, D:\, etc.)
        path.len() >= 3
            && path.chars().nth(1) == Some(':')
            && (path.chars().nth(2) == Some('\\') || path.chars().nth(2) == Some('/'))
    }
}

impl Default for RealWSL2Bridge {
    fn default() -> Self {
        Self::new()
    }
}

impl WSL2Bridge for RealWSL2Bridge {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        // Translate any Windows paths in arguments
        let translated_args = self.translate_args(args)?;

        // Build wsl.exe command
        let output = Command::new("wsl")
            .arg(command)
            .args(&translated_args)
            .output()
            .map_err(|e| anyhow!("Failed to execute WSL command: {}", e))?;

        Ok(CommandOutput::new(
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            output.status.code().unwrap_or(-1),
        ))
    }

    fn is_available(&self) -> bool {
        Command::new("wsl")
            .arg("--version")
            .output()
            .is_ok()
    }

    fn version(&self) -> Result<String> {
        let output = Command::new("wsl")
            .arg("--version")
            .output()
            .map_err(|e| anyhow!("Failed to get WSL version: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(anyhow!("WSL2 is not available"))
        }
    }

    fn translate_path_to_wsl(&self, windows_path: &str) -> Result<String> {
        self.path_translator.to_linux(windows_path)
    }

    fn translate_path_to_windows(&self, wsl_path: &str) -> Result<String> {
        self.path_translator.to_windows(wsl_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_bridge_new() {
        let _bridge = RealWSL2Bridge::new();
        // Just verify it constructs successfully
        assert!(true);
    }

    #[test]
    fn test_real_bridge_default() {
        let _bridge = RealWSL2Bridge::default();
        // Just verify it constructs successfully
        assert!(true);
    }

    #[test]
    fn test_is_windows_path() {
        let bridge = RealWSL2Bridge::new();

        // Windows paths
        assert!(bridge.is_windows_path("C:\\Users\\John"));
        assert!(bridge.is_windows_path("D:\\Projects"));
        assert!(bridge.is_windows_path("C:/Users/John")); // Forward slash variant

        // Not Windows paths
        assert!(!bridge.is_windows_path("/usr/bin"));
        assert!(!bridge.is_windows_path("/mnt/c/Users"));
        assert!(!bridge.is_windows_path("relative/path"));
        assert!(!bridge.is_windows_path("C:"));  // Too short
    }

    #[test]
    fn test_translate_args_no_paths() {
        let bridge = RealWSL2Bridge::new();

        let args = vec!["search", "firefox", "--json"];
        let translated = bridge.translate_args(&args).unwrap();

        assert_eq!(translated, vec!["search", "firefox", "--json"]);
    }

    #[test]
    fn test_translate_args_with_windows_path() {
        let bridge = RealWSL2Bridge::new();

        let args = vec!["build", "-f", "C:\\Users\\John\\config.nix"];
        let translated = bridge.translate_args(&args).unwrap();

        assert_eq!(translated[0], "build");
        assert_eq!(translated[1], "-f");
        assert_eq!(translated[2], "/mnt/c/Users/John/config.nix");
    }

    #[test]
    fn test_path_translation_integration() {
        let bridge = RealWSL2Bridge::new();

        // Windows to WSL
        let wsl_path = bridge.translate_path_to_wsl("C:\\Users\\John").unwrap();
        assert_eq!(wsl_path, "/mnt/c/Users/John");

        // WSL to Windows
        let windows_path = bridge.translate_path_to_windows("/mnt/c/Users/John").unwrap();
        assert_eq!(windows_path, "C:\\Users\\John");
    }

    // Note: We can't reliably test is_available() and version() in CI
    // because WSL2 might not be installed. These are integration tests
    // that should be run manually on Windows.
}
