/// Configuration management for NSFW
///
/// Supports user preferences via ~/.nswfrc file in TOML format
use anyhow::{Result, Context, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs::home_dir;

use crate::templates::WrapperType;

/// User configuration for NSFW
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Cache time-to-live in days (default: 7)
    #[serde(default = "default_cache_ttl_days")]
    pub cache_ttl_days: u32,

    /// Default wrapper type for installed packages
    #[serde(default = "default_wrapper_type")]
    pub default_wrapper_type: String,

    /// Automatically update Nix channels before operations
    #[serde(default = "default_auto_update_channels")]
    pub auto_update_channels: bool,

    /// Installation location for wrapper scripts
    #[serde(default)]
    pub install_location: Option<String>,

    /// Enable verbose output
    #[serde(default = "default_verbose_output")]
    pub verbose_output: bool,

    /// Disable colored output
    #[serde(default = "default_disable_colors")]
    pub disable_colors: bool,

    /// Number of parallel operations for batch commands
    #[serde(default = "default_parallel_jobs")]
    pub parallel_jobs: usize,

    /// Maximum cache size in MB (0 = unlimited)
    #[serde(default = "default_max_cache_size_mb")]
    pub max_cache_size_mb: u64,
}

// Default value functions
fn default_cache_ttl_days() -> u32 { 7 }
fn default_wrapper_type() -> String { "console".to_string() }
fn default_auto_update_channels() -> bool { false }
fn default_verbose_output() -> bool { false }
fn default_disable_colors() -> bool { false }
fn default_parallel_jobs() -> usize { 4 }
fn default_max_cache_size_mb() -> u64 { 100 }

impl Default for Config {
    fn default() -> Self {
        Self {
            cache_ttl_days: default_cache_ttl_days(),
            default_wrapper_type: default_wrapper_type(),
            auto_update_channels: default_auto_update_channels(),
            install_location: None,
            verbose_output: default_verbose_output(),
            disable_colors: default_disable_colors(),
            parallel_jobs: default_parallel_jobs(),
            max_cache_size_mb: default_max_cache_size_mb(),
        }
    }
}

impl Config {
    /// Get the config file path (~/.nswfrc)
    pub fn config_path() -> Result<PathBuf> {
        let home = home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".nswfrc"))
    }

    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        toml::from_str(&content)
            .context("Failed to parse config file")
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Create default config file if it doesn't exist
    pub fn ensure_exists() -> Result<PathBuf> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
        }

        Ok(config_path)
    }

    /// Get a config value by key
    pub fn get(&self, key: &str) -> Result<String> {
        match key {
            "cache_ttl_days" => Ok(self.cache_ttl_days.to_string()),
            "default_wrapper_type" => Ok(self.default_wrapper_type.clone()),
            "auto_update_channels" => Ok(self.auto_update_channels.to_string()),
            "install_location" => Ok(self.install_location.clone().unwrap_or_else(|| "None".to_string())),
            "verbose_output" => Ok(self.verbose_output.to_string()),
            "disable_colors" => Ok(self.disable_colors.to_string()),
            "parallel_jobs" => Ok(self.parallel_jobs.to_string()),
            "max_cache_size_mb" => Ok(self.max_cache_size_mb.to_string()),
            _ => Err(anyhow!("Unknown config key: {}", key)),
        }
    }

    /// Set a config value by key
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "cache_ttl_days" => {
                self.cache_ttl_days = value.parse()
                    .context("Invalid value for cache_ttl_days (must be a positive number)")?;
            }
            "default_wrapper_type" => {
                // Validate wrapper type
                if !["console", "gui", "vbs"].contains(&value) {
                    return Err(anyhow!("Invalid wrapper type. Must be: console, gui, or vbs"));
                }
                self.default_wrapper_type = value.to_string();
            }
            "auto_update_channels" => {
                self.auto_update_channels = value.parse()
                    .context("Invalid value for auto_update_channels (must be true or false)")?;
            }
            "install_location" => {
                if value == "None" || value.is_empty() {
                    self.install_location = None;
                } else {
                    self.install_location = Some(value.to_string());
                }
            }
            "verbose_output" => {
                self.verbose_output = value.parse()
                    .context("Invalid value for verbose_output (must be true or false)")?;
            }
            "disable_colors" => {
                self.disable_colors = value.parse()
                    .context("Invalid value for disable_colors (must be true or false)")?;
            }
            "parallel_jobs" => {
                let jobs: usize = value.parse()
                    .context("Invalid value for parallel_jobs (must be a positive number)")?;
                if jobs == 0 {
                    return Err(anyhow!("parallel_jobs must be at least 1"));
                }
                self.parallel_jobs = jobs;
            }
            "max_cache_size_mb" => {
                self.max_cache_size_mb = value.parse()
                    .context("Invalid value for max_cache_size_mb (must be a positive number)")?;
            }
            _ => {
                return Err(anyhow!("Unknown config key: {}", key));
            }
        }
        Ok(())
    }

    /// Get the default wrapper type as enum
    pub fn get_default_wrapper_type(&self) -> Result<WrapperType> {
        match self.default_wrapper_type.as_str() {
            "console" => Ok(WrapperType::Console),
            "gui" => Ok(WrapperType::Gui),
            "vbs" => Ok(WrapperType::Vbs),
            _ => Err(anyhow!("Invalid default_wrapper_type in config: {}", self.default_wrapper_type)),
        }
    }

    /// Get all config keys
    pub fn keys() -> Vec<&'static str> {
        vec![
            "cache_ttl_days",
            "default_wrapper_type",
            "auto_update_channels",
            "install_location",
            "verbose_output",
            "disable_colors",
            "parallel_jobs",
            "max_cache_size_mb",
        ]
    }

    /// Get description for a config key
    pub fn key_description(key: &str) -> &'static str {
        match key {
            "cache_ttl_days" => "Number of days before package cache expires",
            "default_wrapper_type" => "Default wrapper type for installed packages (console/gui/vbs)",
            "auto_update_channels" => "Automatically update Nix channels before operations",
            "install_location" => "Custom installation location for wrapper scripts",
            "verbose_output" => "Enable verbose output for all commands",
            "disable_colors" => "Disable colored terminal output",
            "parallel_jobs" => "Number of parallel operations for batch commands",
            "max_cache_size_mb" => "Maximum cache size in MB (0 = unlimited)",
            _ => "Unknown configuration key",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.cache_ttl_days, 7);
        assert_eq!(config.default_wrapper_type, "console");
        assert!(!config.auto_update_channels);
        assert!(!config.verbose_output);
        assert!(!config.disable_colors);
        assert_eq!(config.parallel_jobs, 4);
        assert_eq!(config.max_cache_size_mb, 100);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("cache_ttl_days"));
        assert!(toml_str.contains("default_wrapper_type"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            cache_ttl_days = 14
            default_wrapper_type = "gui"
            auto_update_channels = true
            verbose_output = true
            disable_colors = false
            parallel_jobs = 8
            max_cache_size_mb = 200
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.cache_ttl_days, 14);
        assert_eq!(config.default_wrapper_type, "gui");
        assert!(config.auto_update_channels);
        assert!(config.verbose_output);
        assert!(!config.disable_colors);
        assert_eq!(config.parallel_jobs, 8);
        assert_eq!(config.max_cache_size_mb, 200);
    }

    #[test]
    fn test_config_get() {
        let config = Config::default();
        assert_eq!(config.get("cache_ttl_days").unwrap(), "7");
        assert_eq!(config.get("default_wrapper_type").unwrap(), "console");
        assert_eq!(config.get("auto_update_channels").unwrap(), "false");
    }

    #[test]
    fn test_config_get_invalid_key() {
        let config = Config::default();
        assert!(config.get("invalid_key").is_err());
    }

    #[test]
    fn test_config_set() {
        let mut config = Config::default();

        config.set("cache_ttl_days", "14").unwrap();
        assert_eq!(config.cache_ttl_days, 14);

        config.set("default_wrapper_type", "gui").unwrap();
        assert_eq!(config.default_wrapper_type, "gui");

        config.set("auto_update_channels", "true").unwrap();
        assert!(config.auto_update_channels);

        config.set("parallel_jobs", "8").unwrap();
        assert_eq!(config.parallel_jobs, 8);
    }

    #[test]
    fn test_config_set_invalid_wrapper_type() {
        let mut config = Config::default();
        assert!(config.set("default_wrapper_type", "invalid").is_err());
    }

    #[test]
    fn test_config_set_invalid_bool() {
        let mut config = Config::default();
        assert!(config.set("auto_update_channels", "maybe").is_err());
    }

    #[test]
    fn test_config_set_invalid_number() {
        let mut config = Config::default();
        assert!(config.set("cache_ttl_days", "not_a_number").is_err());
    }

    #[test]
    fn test_config_set_zero_parallel_jobs() {
        let mut config = Config::default();
        assert!(config.set("parallel_jobs", "0").is_err());
    }

    #[test]
    fn test_get_default_wrapper_type() {
        let mut config = Config::default();
        assert!(matches!(config.get_default_wrapper_type().unwrap(), WrapperType::Console));

        config.default_wrapper_type = "gui".to_string();
        assert!(matches!(config.get_default_wrapper_type().unwrap(), WrapperType::Gui));

        config.default_wrapper_type = "vbs".to_string();
        assert!(matches!(config.get_default_wrapper_type().unwrap(), WrapperType::Vbs));
    }

    #[test]
    fn test_config_keys() {
        let keys = Config::keys();
        assert!(keys.contains(&"cache_ttl_days"));
        assert!(keys.contains(&"default_wrapper_type"));
        assert!(keys.contains(&"auto_update_channels"));
        assert!(keys.contains(&"parallel_jobs"));
    }

    #[test]
    fn test_key_descriptions() {
        assert!(!Config::key_description("cache_ttl_days").is_empty());
        assert!(!Config::key_description("default_wrapper_type").is_empty());
        assert!(!Config::key_description("parallel_jobs").is_empty());
    }
}
