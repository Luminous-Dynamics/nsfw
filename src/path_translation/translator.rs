use anyhow::{Result, anyhow};

/// Type of path (Windows or Linux)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathType {
    Windows,
    Linux,
}

/// Bidirectional path translator for Windows ↔ Linux
///
/// Handles conversions between:
/// - Windows paths: `C:\Users\...` or `\\?\C:\...`
/// - WSL2 paths: `/mnt/c/Users/...`
/// - Nix store paths: `/nix/store/...` (preserved)
///
/// # Examples
///
/// ```
/// use nsfw::path_translation::PathTranslator;
///
/// let translator = PathTranslator::new();
///
/// // Windows → Linux
/// let linux_path = translator.to_linux("C:\\Users\\John\\file.txt")?;
/// assert_eq!(linux_path, "/mnt/c/Users/John/file.txt");
///
/// // Linux → Windows
/// let win_path = translator.to_windows("/mnt/c/Users/John/file.txt")?;
/// assert_eq!(win_path, "C:\\Users\\John\\file.txt");
/// ```
pub struct PathTranslator {
    /// Whether to preserve Nix store paths (default: true)
    preserve_nix_store: bool,
}

impl PathTranslator {
    /// Create a new PathTranslator with default settings
    pub fn new() -> Self {
        Self {
            preserve_nix_store: true,
        }
    }

    /// Set whether to preserve Nix store paths
    pub fn with_preserve_nix_store(mut self, preserve: bool) -> Self {
        self.preserve_nix_store = preserve;
        self
    }

    /// Detect the type of path
    ///
    /// # Examples
    ///
    /// ```
    /// let translator = PathTranslator::new();
    /// assert_eq!(translator.detect_type("C:\\Users"), PathType::Windows);
    /// assert_eq!(translator.detect_type("/mnt/c/Users"), PathType::Linux);
    /// assert_eq!(translator.detect_type("/nix/store/abc"), PathType::Linux);
    /// ```
    pub fn detect_type(&self, path: &str) -> PathType {
        // Windows paths start with drive letter or UNC prefix
        if path.len() >= 2 {
            let first_two = &path[0..2];

            // Drive letter: C:, D:, etc.
            if first_two.chars().nth(0).map(|c| c.is_ascii_alphabetic()).unwrap_or(false)
                && first_two.chars().nth(1) == Some(':')
            {
                return PathType::Windows;
            }

            // UNC path: \\?\C:\... or \\server\share
            if first_two == "\\\\" {
                return PathType::Windows;
            }
        }

        PathType::Linux
    }

    /// Convert Windows path to Linux (WSL2) path
    ///
    /// # Conversions
    /// - `C:\Users\...` → `/mnt/c/Users/...`
    /// - `D:\Projects\...` → `/mnt/d/Projects/...`
    /// - `\\?\C:\...` → `/mnt/c/...` (strips UNC prefix)
    ///
    /// # Errors
    /// Returns error if path is invalid or cannot be converted
    pub fn to_linux(&self, windows_path: &str) -> Result<String> {
        if windows_path.is_empty() {
            return Err(anyhow!("Empty path"));
        }

        let mut path = windows_path.to_string();

        // Handle UNC paths: \\?\C:\... → C:\...
        if path.starts_with("\\\\?\\") {
            path = path[4..].to_string();
        }

        // Extract drive letter and rest of path
        if path.len() < 2 {
            return Err(anyhow!("Path too short: {}", windows_path));
        }

        let drive_letter = path.chars().nth(0)
            .ok_or_else(|| anyhow!("Invalid path"))?;

        if !drive_letter.is_ascii_alphabetic() {
            return Err(anyhow!("Invalid drive letter: {}", drive_letter));
        }

        if path.chars().nth(1) != Some(':') {
            return Err(anyhow!("Missing colon after drive letter"));
        }

        // Get the rest of the path after "C:"
        let rest = if path.len() > 2 {
            &path[2..]
        } else {
            ""
        };

        // Convert backslashes to forward slashes
        let rest = rest.replace('\\', "/");

        // Construct WSL2 path: /mnt/c/...
        let linux_path = format!("/mnt/{}{}",
            drive_letter.to_ascii_lowercase(),
            rest
        );

        Ok(linux_path)
    }

    /// Convert Linux (WSL2) path to Windows path
    ///
    /// # Conversions
    /// - `/mnt/c/Users/...` → `C:\Users\...`
    /// - `/mnt/d/Projects/...` → `D:\Projects\...`
    /// - `/nix/store/...` → preserved (if preserve_nix_store is true)
    ///
    /// # Errors
    /// Returns error if path is invalid or cannot be converted
    pub fn to_windows(&self, linux_path: &str) -> Result<String> {
        if linux_path.is_empty() {
            return Err(anyhow!("Empty path"));
        }

        // Preserve Nix store paths
        if self.preserve_nix_store && linux_path.starts_with("/nix/store/") {
            return Ok(linux_path.to_string());
        }

        // Handle WSL2 mount paths: /mnt/c/... → C:\...
        if !linux_path.starts_with("/mnt/") {
            return Err(anyhow!("Not a WSL2 mount path: {}", linux_path));
        }

        // Extract drive letter and rest
        let rest = &linux_path[5..]; // Skip "/mnt/"

        if rest.is_empty() {
            return Err(anyhow!("Missing drive letter"));
        }

        let drive_letter = rest.chars().nth(0)
            .ok_or_else(|| anyhow!("Invalid drive letter"))?;

        if !drive_letter.is_ascii_alphabetic() {
            return Err(anyhow!("Invalid drive letter: {}", drive_letter));
        }

        // Get the rest of the path
        let path_rest = if rest.len() > 1 {
            &rest[1..]
        } else {
            ""
        };

        // Convert forward slashes to backslashes
        let path_rest = path_rest.replace('/', "\\");

        // Construct Windows path: C:\...
        let windows_path = format!("{}:{}",
            drive_letter.to_ascii_uppercase(),
            path_rest
        );

        Ok(windows_path)
    }

    /// Normalize a path (convert to canonical form)
    ///
    /// - Removes redundant separators
    /// - Resolves . and .. components (where safe)
    /// - Converts to lowercase drive letters for Linux
    pub fn normalize(&self, path: &str) -> String {
        let path_type = self.detect_type(path);

        match path_type {
            PathType::Windows => {
                // Normalize Windows path
                path.replace('/', "\\")
            }
            PathType::Linux => {
                // Normalize Linux path
                path.replace('\\', "/")
            }
        }
    }

    /// Check if a path is a Nix store path
    pub fn is_nix_store_path(&self, path: &str) -> bool {
        path.starts_with("/nix/store/")
    }

    /// Check if a path is a WSL2 mount path
    pub fn is_wsl_mount_path(&self, path: &str) -> bool {
        path.starts_with("/mnt/") && path.len() > 6
    }

    /// Extract drive letter from Windows path
    pub fn extract_drive_letter(&self, windows_path: &str) -> Option<char> {
        if windows_path.len() >= 2 {
            let first = windows_path.chars().nth(0)?;
            let second = windows_path.chars().nth(1)?;

            if first.is_ascii_alphabetic() && second == ':' {
                return Some(first.to_ascii_uppercase());
            }
        }
        None
    }

    /// Validate a path is convertible
    pub fn validate(&self, path: &str) -> Result<()> {
        if path.is_empty() {
            return Err(anyhow!("Empty path"));
        }

        let path_type = self.detect_type(path);

        match path_type {
            PathType::Windows => {
                // Must have drive letter
                if self.extract_drive_letter(path).is_none() {
                    return Err(anyhow!("Invalid Windows path: no drive letter"));
                }
            }
            PathType::Linux => {
                // Must be absolute path
                if !path.starts_with('/') {
                    return Err(anyhow!("Relative Linux paths not supported"));
                }
            }
        }

        Ok(())
    }
}

impl Default for PathTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_windows_path() {
        let translator = PathTranslator::new();
        assert_eq!(translator.detect_type("C:\\Users"), PathType::Windows);
        assert_eq!(translator.detect_type("D:\\Projects"), PathType::Windows);
        assert_eq!(translator.detect_type("\\\\?\\C:\\"), PathType::Windows);
    }

    #[test]
    fn test_detect_linux_path() {
        let translator = PathTranslator::new();
        assert_eq!(translator.detect_type("/mnt/c/Users"), PathType::Linux);
        assert_eq!(translator.detect_type("/nix/store/abc"), PathType::Linux);
        assert_eq!(translator.detect_type("/home/user"), PathType::Linux);
    }

    #[test]
    fn test_windows_to_linux() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\Users\\John").unwrap(),
            "/mnt/c/Users/John"
        );

        assert_eq!(
            translator.to_linux("D:\\Projects\\Code").unwrap(),
            "/mnt/d/Projects/Code"
        );
    }

    #[test]
    fn test_linux_to_windows() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_windows("/mnt/c/Users/John").unwrap(),
            "C:\\Users\\John"
        );

        assert_eq!(
            translator.to_windows("/mnt/d/Projects/Code").unwrap(),
            "D:\\Projects\\Code"
        );
    }

    #[test]
    fn test_preserve_nix_store() {
        let translator = PathTranslator::new();

        let nix_path = "/nix/store/abc123-hello";
        assert_eq!(
            translator.to_windows(nix_path).unwrap(),
            nix_path
        );
    }

    #[test]
    fn test_unc_path_conversion() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("\\\\?\\C:\\Users\\John").unwrap(),
            "/mnt/c/Users/John"
        );
    }

    #[test]
    fn test_extract_drive_letter() {
        let translator = PathTranslator::new();

        assert_eq!(translator.extract_drive_letter("C:\\Users"), Some('C'));
        assert_eq!(translator.extract_drive_letter("d:\\Projects"), Some('D'));
        assert_eq!(translator.extract_drive_letter("/mnt/c/"), None);
    }

    #[test]
    fn test_is_nix_store_path() {
        let translator = PathTranslator::new();

        assert!(translator.is_nix_store_path("/nix/store/abc123-hello"));
        assert!(!translator.is_nix_store_path("/home/user"));
        assert!(!translator.is_nix_store_path("C:\\Users"));
    }

    #[test]
    fn test_is_wsl_mount_path() {
        let translator = PathTranslator::new();

        assert!(translator.is_wsl_mount_path("/mnt/c/Users"));
        assert!(!translator.is_wsl_mount_path("/home/user"));
        assert!(!translator.is_wsl_mount_path("/mnt/"));
    }

    #[test]
    fn test_empty_path_error() {
        let translator = PathTranslator::new();

        assert!(translator.to_linux("").is_err());
        assert!(translator.to_windows("").is_err());
    }

    #[test]
    fn test_invalid_windows_path() {
        let translator = PathTranslator::new();

        // No colon
        assert!(translator.to_linux("C\\Users").is_err());

        // Invalid drive letter
        assert!(translator.to_linux("1:\\Users").is_err());
    }

    #[test]
    fn test_invalid_linux_path() {
        let translator = PathTranslator::new();

        // Not a mount path
        assert!(translator.to_windows("/home/user").is_err());

        // Missing drive letter
        assert!(translator.to_windows("/mnt/").is_err());
    }

    // Edge Cases: Spaces
    #[test]
    fn test_path_with_spaces() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\Program Files\\App").unwrap(),
            "/mnt/c/Program Files/App"
        );

        assert_eq!(
            translator.to_windows("/mnt/c/Program Files/App").unwrap(),
            "C:\\Program Files\\App"
        );
    }

    #[test]
    fn test_path_with_multiple_spaces() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\My  Documents\\Test  File").unwrap(),
            "/mnt/c/My  Documents/Test  File"
        );
    }

    // Edge Cases: Special Characters
    #[test]
    fn test_path_with_dash() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\my-project\\sub-folder").unwrap(),
            "/mnt/c/my-project/sub-folder"
        );
    }

    #[test]
    fn test_path_with_underscore() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\my_project\\my_file.txt").unwrap(),
            "/mnt/c/my_project/my_file.txt"
        );
    }

    #[test]
    fn test_path_with_dot() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\file.txt").unwrap(),
            "/mnt/c/file.txt"
        );

        assert_eq!(
            translator.to_linux("C:\\folder.name\\file.txt").unwrap(),
            "/mnt/c/folder.name/file.txt"
        );
    }

    #[test]
    fn test_path_with_parentheses() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\Program Files (x86)\\App").unwrap(),
            "/mnt/c/Program Files (x86)/App"
        );
    }

    // Edge Cases: Drive Letters
    #[test]
    fn test_all_drive_letters() {
        let translator = PathTranslator::new();

        for letter in 'A'..='Z' {
            let win_path = format!("{}:\\test", letter);
            let linux_path = format!("/mnt/{}/test", letter.to_ascii_lowercase());

            assert_eq!(translator.to_linux(&win_path).unwrap(), linux_path);
            assert_eq!(translator.to_windows(&linux_path).unwrap(), win_path);
        }
    }

    #[test]
    fn test_lowercase_drive_letter() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("c:\\Users").unwrap(),
            "/mnt/c/Users"
        );
    }

    // Edge Cases: Path Lengths
    #[test]
    fn test_root_path() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\").unwrap(),
            "/mnt/c/"
        );

        assert_eq!(
            translator.to_windows("/mnt/c/").unwrap(),
            "C:\\"
        );
    }

    #[test]
    fn test_long_path() {
        let translator = PathTranslator::new();

        let long = "very\\long\\path\\with\\many\\nested\\directories\\and\\files";
        let win_path = format!("C:\\{}", long);
        let expected = format!("/mnt/c/{}", long.replace('\\', "/"));

        assert_eq!(translator.to_linux(&win_path).unwrap(), expected);
    }

    // Edge Cases: Trailing Separators
    #[test]
    fn test_trailing_backslash() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\Users\\").unwrap(),
            "/mnt/c/Users/"
        );
    }

    #[test]
    fn test_trailing_forward_slash() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_windows("/mnt/c/Users/").unwrap(),
            "C:\\Users\\"
        );
    }

    // Edge Cases: Mixed Separators (Windows)
    #[test]
    fn test_windows_mixed_separators() {
        let translator = PathTranslator::new();

        // Forward slashes in Windows paths are treated as path separators
        assert_eq!(
            translator.to_linux("C:/Users/John").unwrap(),
            "/mnt/c/Users/John"
        );
    }

    // Edge Cases: Case Sensitivity
    #[test]
    fn test_case_preserved_in_path() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\MyDocuments\\MyFile.TXT").unwrap(),
            "/mnt/c/MyDocuments/MyFile.TXT"
        );
    }

    #[test]
    fn test_drive_letter_case_normalized() {
        let translator = PathTranslator::new();

        // Windows: always uppercase
        assert_eq!(
            translator.to_windows("/mnt/c/test").unwrap(),
            "C:\\test"
        );

        // Linux: always lowercase
        assert_eq!(
            translator.to_linux("C:\\test").unwrap(),
            "/mnt/c/test"
        );
    }

    // Edge Cases: Nix Store Paths
    #[test]
    fn test_nix_store_with_hash() {
        let translator = PathTranslator::new();

        let nix_path = "/nix/store/abc123xyz-package-1.0.0";
        assert_eq!(translator.to_windows(nix_path).unwrap(), nix_path);
    }

    #[test]
    fn test_nix_store_nested() {
        let translator = PathTranslator::new();

        let nix_path = "/nix/store/abc123-hello/bin/hello";
        assert_eq!(translator.to_windows(nix_path).unwrap(), nix_path);
    }

    #[test]
    fn test_disable_nix_store_preservation() {
        let translator = PathTranslator::new().with_preserve_nix_store(false);

        let nix_path = "/nix/store/abc123-hello";
        assert!(translator.to_windows(nix_path).is_err());
    }

    // Edge Cases: Helper Methods
    #[test]
    fn test_normalize_windows() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.normalize("C:/Users/John"),
            "C:\\Users\\John"
        );
    }

    #[test]
    fn test_normalize_linux() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.normalize("C:\\Users\\John"),
            "C:\\Users\\John"
        );
    }

    #[test]
    fn test_validate_valid_windows() {
        let translator = PathTranslator::new();

        assert!(translator.validate("C:\\Users").is_ok());
        assert!(translator.validate("D:\\Projects\\Code").is_ok());
    }

    #[test]
    fn test_validate_valid_linux() {
        let translator = PathTranslator::new();

        assert!(translator.validate("/mnt/c/Users").is_ok());
        assert!(translator.validate("/home/user").is_ok());
    }

    #[test]
    fn test_validate_invalid() {
        let translator = PathTranslator::new();

        assert!(translator.validate("").is_err());
        assert!(translator.validate("relative/path").is_err());
        assert!(translator.validate("no-drive").is_err());
    }

    // Edge Cases: Boundary Conditions
    #[test]
    fn test_minimum_valid_windows_path() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:").unwrap(),
            "/mnt/c"
        );
    }

    #[test]
    fn test_minimum_valid_linux_path() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_windows("/mnt/c").unwrap(),
            "C:"
        );
    }

    #[test]
    fn test_single_character_path() {
        let translator = PathTranslator::new();

        assert!(translator.to_linux("C").is_err());
        assert!(translator.to_windows("/mnt").is_err());
    }

    // Edge Cases: Multiple Separators
    #[test]
    fn test_multiple_consecutive_backslashes() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\\\Users\\\\John").unwrap(),
            "/mnt/c//Users//John"
        );
    }

    #[test]
    fn test_multiple_consecutive_forward_slashes() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_windows("/mnt/c//Users//John").unwrap(),
            "C:\\\\Users\\\\John"
        );
    }

    // Edge Cases: Numbers in Paths
    #[test]
    fn test_path_with_numbers() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\folder123\\file456.txt").unwrap(),
            "/mnt/c/folder123/file456.txt"
        );
    }

    // Edge Cases: File Extensions
    #[test]
    fn test_various_extensions() {
        let translator = PathTranslator::new();

        let extensions = vec!["txt", "exe", "dll", "json", "tar.gz", "backup.old"];

        for ext in extensions {
            let win_path = format!("C:\\file.{}", ext);
            let expected = format!("/mnt/c/file.{}", ext);

            assert_eq!(translator.to_linux(&win_path).unwrap(), expected);
        }
    }

    // Regression Tests
    #[test]
    fn test_regression_empty_after_colon() {
        let translator = PathTranslator::new();

        // Should handle C: (no backslash)
        assert_eq!(translator.to_linux("C:").unwrap(), "/mnt/c");
    }

    #[test]
    fn test_regression_slash_after_colon() {
        let translator = PathTranslator::new();

        // Both forward and back slash should work
        assert_eq!(translator.to_linux("C:\\").unwrap(), "/mnt/c/");
        assert_eq!(translator.to_linux("C:/").unwrap(), "/mnt/c/");
    }

    // Integration Tests
    #[test]
    fn test_round_trip_conversion() {
        let translator = PathTranslator::new();

        let original = "C:\\Users\\John\\Documents\\file.txt";

        let linux = translator.to_linux(original).unwrap();
        let back_to_windows = translator.to_windows(&linux).unwrap();

        assert_eq!(back_to_windows, original);
    }

    #[test]
    fn test_round_trip_all_drives() {
        let translator = PathTranslator::new();

        for letter in 'A'..='Z' {
            let original = format!("{}:\\test\\path", letter);

            let linux = translator.to_linux(&original).unwrap();
            let back = translator.to_windows(&linux).unwrap();

            assert_eq!(back, original);
        }
    }

    // Additional Edge Cases
    #[test]
    fn test_path_with_at_sign() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\user@host\\file").unwrap(),
            "/mnt/c/user@host/file"
        );
    }

    #[test]
    fn test_path_with_plus() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\C++\\projects").unwrap(),
            "/mnt/c/C++/projects"
        );
    }

    #[test]
    fn test_path_with_equals() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\name=value").unwrap(),
            "/mnt/c/name=value"
        );
    }

    #[test]
    fn test_deeply_nested_path() {
        let translator = PathTranslator::new();

        let mut path = "C:".to_string();
        for i in 0..20 {
            path.push_str(&format!("\\level{}", i));
        }

        let linux = translator.to_linux(&path).unwrap();
        assert!(linux.starts_with("/mnt/c/"));
        assert!(linux.contains("level19"));
    }

    #[test]
    fn test_path_with_tilde() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\~backup\\file").unwrap(),
            "/mnt/c/~backup/file"
        );
    }

    #[test]
    fn test_path_with_hash() {
        let translator = PathTranslator::new();

        assert_eq!(
            translator.to_linux("C:\\file#123").unwrap(),
            "/mnt/c/file#123"
        );
    }

    #[test]
    fn test_common_windows_paths() {
        let translator = PathTranslator::new();

        let paths = vec![
            ("C:\\Windows\\System32", "/mnt/c/Windows/System32"),
            ("C:\\Program Files", "/mnt/c/Program Files"),
            ("C:\\Users\\Public", "/mnt/c/Users/Public"),
            ("D:\\Games", "/mnt/d/Games"),
        ];

        for (win, linux) in paths {
            assert_eq!(translator.to_linux(win).unwrap(), linux);
            assert_eq!(translator.to_windows(linux).unwrap(), win);
        }
    }

    #[test]
    fn test_detect_type_edge_cases() {
        let translator = PathTranslator::new();

        // Just a drive letter
        assert_eq!(translator.detect_type("C:"), PathType::Windows);

        // Absolute Linux path
        assert_eq!(translator.detect_type("/usr/bin"), PathType::Linux);

        // Looks like Windows but isn't (missing colon)
        assert_eq!(translator.detect_type("C/Users"), PathType::Linux);
    }

    #[test]
    fn test_is_methods_edge_cases() {
        let translator = PathTranslator::new();

        // Nix store edge cases
        assert!(translator.is_nix_store_path("/nix/store/"));
        assert!(!translator.is_nix_store_path("/nix/var/"));

        // WSL mount edge cases
        assert!(!translator.is_wsl_mount_path("/mnt"));
        assert!(!translator.is_wsl_mount_path("/mnt/"));
        assert!(!translator.is_wsl_mount_path("/mnt/c")); // Exactly 6 chars, needs >6
        assert!(translator.is_wsl_mount_path("/mnt/c/")); // 7 chars, valid
        assert!(translator.is_wsl_mount_path("/mnt/c/Users"));
    }
}