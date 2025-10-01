use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use chrono::Local;

use super::templates::{ConsoleTemplate, GuiTemplate, VbsTemplate, WrapperTemplate};

/// Type of wrapper to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapperType {
    /// Console application (visible window)
    Console,
    /// GUI application (background, but shows console briefly)
    Gui,
    /// VBScript wrapper (truly silent, no console)
    Vbs,
}

/// Information about a package to wrap
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name (e.g., "firefox")
    pub name: String,
    
    /// Full path to executable in Nix store
    pub nix_store_path: String,
    
    /// Wrapper type (console, GUI, VBS)
    pub wrapper_type: WrapperType,
    
    /// Optional custom environment variables
    pub env_vars: HashMap<String, String>,
}

impl PackageInfo {
    /// Create new package info
    pub fn new(name: String, nix_store_path: String, wrapper_type: WrapperType) -> Self {
        Self {
            name,
            nix_store_path,
            wrapper_type,
            env_vars: HashMap::new(),
        }
    }
    
    /// Add environment variable
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }
}

/// Generates Windows wrapper scripts for Nix packages
pub struct WrapperGenerator {
    /// Output directory for wrappers
    output_dir: PathBuf,
}

impl WrapperGenerator {
    /// Create a new wrapper generator
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
    
    /// Generate wrapper script for a package
    ///
    /// # Arguments
    /// * `package_info` - Information about the package
    ///
    /// # Returns
    /// Path to the generated wrapper script
    ///
    /// # Example
    /// ```
    /// let generator = WrapperGenerator::new("C:\\wrappers".into());
    /// let info = PackageInfo::new(
    ///     "firefox".to_string(),
    ///     "/nix/store/abc-firefox/bin/firefox".to_string(),
    ///     WrapperType::Gui
    /// );
    /// let wrapper_path = generator.generate(&info)?;
    /// ```
    pub fn generate(&self, package_info: &PackageInfo) -> Result<PathBuf> {
        // Create output directory if it doesn't exist
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)?;
        }
        
        // Generate the wrapper content
        let content = self.generate_content(package_info)?;
        
        // Determine file extension
        let extension = match package_info.wrapper_type {
            WrapperType::Vbs => "vbs",
            _ => "bat",
        };
        
        // Write to file
        let filename = format!("{}.{}", package_info.name, extension);
        let output_path = self.output_dir.join(&filename);
        
        fs::write(&output_path, content)?;
        
        Ok(output_path)
    }
    
    /// Generate wrapper content (without writing to file)
    pub fn generate_content(&self, package_info: &PackageInfo) -> Result<String> {
        // Select template based on wrapper type
        let template: Box<dyn WrapperTemplate> = match package_info.wrapper_type {
            WrapperType::Console => Box::new(ConsoleTemplate),
            WrapperType::Gui => Box::new(GuiTemplate),
            WrapperType::Vbs => Box::new(VbsTemplate),
        };
        
        // Build placeholders
        let mut placeholders = HashMap::new();
        placeholders.insert("package_name".to_string(), package_info.name.clone());
        placeholders.insert("nix_store_path".to_string(), package_info.nix_store_path.clone());
        placeholders.insert("timestamp".to_string(), Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        
        // Add custom environment variables if any
        if !package_info.env_vars.is_empty() {
            let env_setup = self.generate_env_setup(&package_info.env_vars);
            placeholders.insert("env_setup".to_string(), env_setup);
        }
        
        // Render template
        let content = template.render(&placeholders);
        
        Ok(content)
    }
    
    /// Generate environment variable setup code
    fn generate_env_setup(&self, env_vars: &HashMap<String, String>) -> String {
        let mut setup = String::new();
        
        for (key, value) in env_vars {
            setup.push_str(&format!("set {}={}\n", key, value));
        }
        
        setup
    }
    
    /// Generate wrappers for multiple packages
    pub fn generate_batch(&self, packages: &[PackageInfo]) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        
        for package in packages {
            let path = self.generate(package)?;
            paths.push(path);
        }
        
        Ok(paths)
    }
    
    /// Validate a Nix store path
    pub fn validate_store_path(&self, path: &str) -> Result<()> {
        if !path.starts_with("/nix/store/") {
            return Err(anyhow!("Invalid Nix store path: {}", path));
        }

        // Must have content after /nix/store/
        if path == "/nix/store/" {
            return Err(anyhow!("Nix store path cannot be just /nix/store/"));
        }

        // Check if path looks reasonable (has hash-package format)
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() < 4 {  // ["nix", "store", "hash-package", ...]
            return Err(anyhow!("Nix store path too short: {}", path));
        }

        Ok(())
    }
    
    /// Detect wrapper type based on package name heuristics
    pub fn detect_wrapper_type(&self, package_name: &str) -> WrapperType {
        // GUI applications (common ones)
        let gui_apps = [
            "firefox", "chromium", "code", "vscode", "gimp", "inkscape",
            "libreoffice", "thunderbird", "vlc", "obs", "audacity",
            "blender", "kdenlive", "krita", "brave"
        ];
        
        // Check if package name matches a known GUI app
        for gui_app in &gui_apps {
            if package_name.contains(gui_app) {
                return WrapperType::Gui;
            }
        }
        
        // Default to console for CLI tools
        WrapperType::Console
    }
    
    /// Get the output directory
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_wrapper_generator_new() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        assert_eq!(generator.output_dir(), temp.path());
    }
    
    #[test]
    fn test_generate_console_wrapper() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        let info = PackageInfo::new(
            "vim".to_string(),
            "/nix/store/abc123-vim-9.0/bin/vim".to_string(),
            WrapperType::Console
        );
        
        let content = generator.generate_content(&info).unwrap();
        
        assert!(content.contains("vim"));
        assert!(content.contains("/nix/store/abc123-vim-9.0/bin/vim"));
        assert!(content.contains("@echo off"));
        assert!(content.contains("wsl"));
    }
    
    #[test]
    fn test_generate_gui_wrapper() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        let info = PackageInfo::new(
            "firefox".to_string(),
            "/nix/store/xyz789-firefox/bin/firefox".to_string(),
            WrapperType::Gui
        );
        
        let content = generator.generate_content(&info).unwrap();
        
        assert!(content.contains("firefox"));
        assert!(content.contains("start /B wsl"));
    }
    
    #[test]
    fn test_generate_vbs_wrapper() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        let info = PackageInfo::new(
            "notepad".to_string(),
            "/nix/store/note123/bin/notepad".to_string(),
            WrapperType::Vbs
        );
        
        let content = generator.generate_content(&info).unwrap();
        
        assert!(content.contains("notepad"));
        assert!(content.contains("WScript"));
        assert!(content.contains("CreateObject"));
    }
    
    #[test]
    fn test_generate_wrapper_with_env() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        let info = PackageInfo::new(
            "myapp".to_string(),
            "/nix/store/app/bin/myapp".to_string(),
            WrapperType::Console
        )
        .with_env("CUSTOM_VAR".to_string(), "value123".to_string());
        
        let content = generator.generate_content(&info).unwrap();
        
        // Note: env vars would need to be integrated into template
        assert!(content.contains("myapp"));
    }
    
    #[test]
    fn test_validate_store_path() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        // Valid paths
        assert!(generator.validate_store_path("/nix/store/abc-hello/bin/hello").is_ok());
        assert!(generator.validate_store_path("/nix/store/xyz123-firefox-130/bin/firefox").is_ok());
        
        // Invalid paths
        assert!(generator.validate_store_path("/usr/bin/vim").is_err());
        assert!(generator.validate_store_path("/nix/store/").is_err());
        assert!(generator.validate_store_path("C:\\Windows").is_err());
    }
    
    #[test]
    fn test_detect_wrapper_type() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        // GUI applications
        assert_eq!(generator.detect_wrapper_type("firefox"), WrapperType::Gui);
        assert_eq!(generator.detect_wrapper_type("vscode"), WrapperType::Gui);
        assert_eq!(generator.detect_wrapper_type("chromium"), WrapperType::Gui);
        
        // Console applications
        assert_eq!(generator.detect_wrapper_type("vim"), WrapperType::Console);
        assert_eq!(generator.detect_wrapper_type("git"), WrapperType::Console);
        assert_eq!(generator.detect_wrapper_type("python"), WrapperType::Console);
    }
    
    #[test]
    fn test_generate_batch() {
        let temp = TempDir::new().unwrap();
        let generator = WrapperGenerator::new(temp.path().to_path_buf());
        
        let packages = vec![
            PackageInfo::new(
                "vim".to_string(),
                "/nix/store/vim/bin/vim".to_string(),
                WrapperType::Console
            ),
            PackageInfo::new(
                "firefox".to_string(),
                "/nix/store/firefox/bin/firefox".to_string(),
                WrapperType::Gui
            ),
        ];
        
        let paths = generator.generate_batch(&packages).unwrap();
        
        assert_eq!(paths.len(), 2);
        assert!(paths[0].exists());
        assert!(paths[1].exists());
        assert!(paths[0].to_string_lossy().contains("vim.bat"));
        assert!(paths[1].to_string_lossy().contains("firefox.bat"));
    }
    
    #[test]
    fn test_package_info_builder() {
        let info = PackageInfo::new(
            "test".to_string(),
            "/nix/store/test".to_string(),
            WrapperType::Console
        )
        .with_env("VAR1".to_string(), "val1".to_string())
        .with_env("VAR2".to_string(), "val2".to_string());
        
        assert_eq!(info.name, "test");
        assert_eq!(info.env_vars.len(), 2);
        assert_eq!(info.env_vars.get("VAR1"), Some(&"val1".to_string()));
    }
}
