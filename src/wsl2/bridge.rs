/// WSL2 bridge trait for executing commands in WSL2
///
/// This trait provides an abstraction over WSL2 command execution,
/// allowing for both real WSL2 execution and mock testing.
use anyhow::Result;

/// Output from a command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Standard output from the command
    pub stdout: String,

    /// Standard error from the command
    pub stderr: String,

    /// Exit code from the command
    pub exit_code: i32,
}

impl CommandOutput {
    /// Create a new command output
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            exit_code,
        }
    }

    /// Check if the command succeeded (exit code 0)
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Check if the command failed (non-zero exit code)
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

/// WSL2 bridge trait
///
/// Provides methods for executing commands in WSL2 and translating paths
pub trait WSL2Bridge {
    /// Execute a command in WSL2
    ///
    /// # Arguments
    /// * `command` - The command to execute (e.g., "nix", "ls")
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// CommandOutput with stdout, stderr, and exit code
    ///
    /// # Example
    /// ```
    /// let output = bridge.execute("nix", &["--version"])?;
    /// println!("Nix version: {}", output.stdout);
    /// ```
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput>;

    /// Check if WSL2 is available
    ///
    /// # Returns
    /// `true` if WSL2 is available and working, `false` otherwise
    fn is_available(&self) -> bool;

    /// Get WSL2 version information
    ///
    /// # Returns
    /// Version string if WSL2 is available
    ///
    /// # Example
    /// ```
    /// let version = bridge.version()?;
    /// println!("WSL2 version: {}", version);
    /// ```
    fn version(&self) -> Result<String>;

    /// Translate a Windows path to WSL2 path format
    ///
    /// # Arguments
    /// * `windows_path` - Windows-style path (e.g., "C:\\Users\\John")
    ///
    /// # Returns
    /// WSL2-style path (e.g., "/mnt/c/Users/John")
    ///
    /// # Example
    /// ```
    /// let wsl_path = bridge.translate_path_to_wsl("C:\\Users\\John")?;
    /// assert_eq!(wsl_path, "/mnt/c/Users/John");
    /// ```
    fn translate_path_to_wsl(&self, windows_path: &str) -> Result<String>;

    /// Translate a WSL2 path to Windows path format
    ///
    /// # Arguments
    /// * `wsl_path` - WSL2-style path (e.g., "/mnt/c/Users/John")
    ///
    /// # Returns
    /// Windows-style path (e.g., "C:\\Users\\John")
    ///
    /// # Example
    /// ```
    /// let windows_path = bridge.translate_path_to_windows("/mnt/c/Users/John")?;
    /// assert_eq!(windows_path, "C:\\Users\\John");
    /// ```
    fn translate_path_to_windows(&self, wsl_path: &str) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_output_new() {
        let output = CommandOutput::new(
            "hello".to_string(),
            "".to_string(),
            0
        );

        assert_eq!(output.stdout, "hello");
        assert_eq!(output.stderr, "");
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn test_command_output_is_success() {
        let success = CommandOutput::new("".to_string(), "".to_string(), 0);
        assert!(success.is_success());
        assert!(!success.is_failure());

        let failure = CommandOutput::new("".to_string(), "error".to_string(), 1);
        assert!(!failure.is_success());
        assert!(failure.is_failure());
    }
}
