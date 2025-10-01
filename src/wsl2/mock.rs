/// Mock WSL2 bridge for testing
use std::collections::HashMap;
use anyhow::{Result, anyhow};

use super::bridge::{WSL2Bridge, CommandOutput};
use crate::path_translation::PathTranslator;

/// Mock WSL2 bridge for testing without actual WSL2
///
/// This allows testing all WSL2-dependent code on any platform,
/// including CI/CD environments without WSL2.
pub struct MockWSL2Bridge {
    /// Predefined responses for commands
    responses: HashMap<String, CommandOutput>,

    /// Whether WSL2 should be considered available
    available: bool,

    /// WSL2 version to report
    version: String,

    /// Path translator for path conversion
    path_translator: PathTranslator,
}

impl MockWSL2Bridge {
    /// Create a new mock bridge with default settings
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            available: true,
            version: "WSL version: 2.0.0.0 (Mock)".to_string(),
            path_translator: PathTranslator::new(),
        }
    }

    /// Set a mock response for a specific command
    ///
    /// # Arguments
    /// * `command` - Full command with args (e.g., "nix --version")
    /// * `output` - The mock output to return
    ///
    /// # Example
    /// ```
    /// let mut bridge = MockWSL2Bridge::new();
    /// bridge.set_response(
    ///     "nix --version".to_string(),
    ///     CommandOutput::new("nix 2.18.1".to_string(), "".to_string(), 0)
    /// );
    /// ```
    pub fn set_response(&mut self, command: String, output: CommandOutput) {
        self.responses.insert(command, output);
    }

    /// Set whether WSL2 should be considered available
    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }

    /// Set the WSL2 version string to report
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Build full command string from command and args
    fn build_command_string(&self, command: &str, args: &[&str]) -> String {
        if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        }
    }

    /// Add common Nix command responses
    ///
    /// This is a helper to set up common responses for testing:
    /// - nix --version
    /// - nix search results
    /// - nix profile list results
    pub fn add_common_responses(&mut self) {
        // Nix version
        self.set_response(
            "nix --version".to_string(),
            CommandOutput::new("nix (Nix) 2.18.1".to_string(), "".to_string(), 0)
        );

        // Empty search result
        self.set_response(
            "nix search nixpkgs test --json".to_string(),
            CommandOutput::new("{}".to_string(), "".to_string(), 0)
        );

        // Empty profile list
        self.set_response(
            "nix profile list --json".to_string(),
            CommandOutput::new(r#"{"elements":{}}"#.to_string(), "".to_string(), 0)
        );
    }
}

impl Default for MockWSL2Bridge {
    fn default() -> Self {
        let mut bridge = Self::new();
        bridge.add_common_responses();
        bridge
    }
}

impl WSL2Bridge for MockWSL2Bridge {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        if !self.available {
            return Err(anyhow!("WSL2 is not available (mock)"));
        }

        let full_command = self.build_command_string(command, args);

        self.responses
            .get(&full_command)
            .cloned()
            .ok_or_else(|| anyhow!("No mock response configured for command: '{}'", full_command))
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn version(&self) -> Result<String> {
        if self.available {
            Ok(self.version.clone())
        } else {
            Err(anyhow!("WSL2 is not available (mock)"))
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
    fn test_mock_bridge_new() {
        let bridge = MockWSL2Bridge::new();
        assert!(bridge.is_available());
    }

    #[test]
    fn test_mock_bridge_default() {
        let bridge = MockWSL2Bridge::default();
        assert!(bridge.is_available());

        // Should have common responses
        let version = bridge.execute("nix", &["--version"]).unwrap();
        assert!(version.is_success());
        assert!(version.stdout.contains("nix"));
    }

    #[test]
    fn test_set_response() {
        let mut bridge = MockWSL2Bridge::new();

        bridge.set_response(
            "echo hello".to_string(),
            CommandOutput::new("hello\n".to_string(), "".to_string(), 0)
        );

        let output = bridge.execute("echo", &["hello"]).unwrap();
        assert_eq!(output.stdout, "hello\n");
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn test_set_available() {
        let mut bridge = MockWSL2Bridge::new();

        bridge.set_available(false);
        assert!(!bridge.is_available());

        let result = bridge.execute("nix", &["--version"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not available"));

        bridge.set_available(true);
        assert!(bridge.is_available());
    }

    #[test]
    fn test_set_version() {
        let mut bridge = MockWSL2Bridge::new();

        bridge.set_version("WSL version: 3.0.0.0 (Test)".to_string());
        let version = bridge.version().unwrap();
        assert_eq!(version, "WSL version: 3.0.0.0 (Test)");
    }

    #[test]
    fn test_command_not_found() {
        let bridge = MockWSL2Bridge::new();

        let result = bridge.execute("unknown", &["command"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No mock response"));
    }

    #[test]
    fn test_path_translation() {
        let bridge = MockWSL2Bridge::new();

        // Windows to WSL
        let wsl_path = bridge.translate_path_to_wsl("C:\\Users\\John").unwrap();
        assert_eq!(wsl_path, "/mnt/c/Users/John");

        // WSL to Windows
        let windows_path = bridge.translate_path_to_windows("/mnt/c/Users/John").unwrap();
        assert_eq!(windows_path, "C:\\Users\\John");
    }

    #[test]
    fn test_add_common_responses() {
        let mut bridge = MockWSL2Bridge::new();
        bridge.add_common_responses();

        // Should have nix version
        let version = bridge.execute("nix", &["--version"]).unwrap();
        assert!(version.stdout.contains("nix"));

        // Should have empty search
        let search = bridge.execute("nix", &["search", "nixpkgs", "test", "--json"]).unwrap();
        assert_eq!(search.stdout, "{}");

        // Should have empty profile list
        let list = bridge.execute("nix", &["profile", "list", "--json"]).unwrap();
        assert!(list.stdout.contains("elements"));
    }

    #[test]
    fn test_build_command_string() {
        let bridge = MockWSL2Bridge::new();

        assert_eq!(
            bridge.build_command_string("nix", &[]),
            "nix"
        );

        assert_eq!(
            bridge.build_command_string("nix", &["--version"]),
            "nix --version"
        );

        assert_eq!(
            bridge.build_command_string("nix", &["search", "nixpkgs", "firefox"]),
            "nix search nixpkgs firefox"
        );
    }

    #[test]
    fn test_mock_error_response() {
        let mut bridge = MockWSL2Bridge::new();

        bridge.set_response(
            "nix install invalid-package".to_string(),
            CommandOutput::new(
                "".to_string(),
                "error: package not found".to_string(),
                1
            )
        );

        let output = bridge.execute("nix", &["install", "invalid-package"]).unwrap();
        assert!(output.is_failure());
        assert_eq!(output.exit_code, 1);
        assert!(output.stderr.contains("not found"));
    }
}
