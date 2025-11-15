use chrono::Local;
use colored::Colorize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Log level for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    /// Get the string representation of the log level
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Get the colored representation of the log level
    pub fn colored(&self, disable_colors: bool) -> String {
        if disable_colors {
            return self.as_str().to_string();
        }

        match self {
            LogLevel::Debug => self.as_str().dimmed().to_string(),
            LogLevel::Info => self.as_str().bright_cyan().to_string(),
            LogLevel::Warn => self.as_str().bright_yellow().to_string(),
            LogLevel::Error => self.as_str().bright_red().bold().to_string(),
        }
    }
}

/// Global logger instance
static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub log_to_file: bool,
    pub log_file_path: Option<PathBuf>,
    pub disable_colors: bool,
    pub include_timestamp: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            log_to_file: false,
            log_file_path: None,
            disable_colors: false,
            include_timestamp: false,
        }
    }
}

/// Logger implementation
pub struct Logger {
    config: LoggerConfig,
}

impl Logger {
    /// Create a new logger with the given configuration
    pub fn new(config: LoggerConfig) -> Self {
        Self { config }
    }

    /// Log a message at the specified level
    pub fn log(&self, level: LogLevel, message: &str) {
        // Check if we should log this message
        if level < self.config.min_level {
            return;
        }

        let timestamp = if self.config.include_timestamp {
            format!("[{}] ", Local::now().format("%Y-%m-%d %H:%M:%S"))
        } else {
            String::new()
        };

        let level_str = level.colored(self.config.disable_colors);
        let formatted = format!("{}{:7} {}", timestamp, level_str, message);

        // Output to stderr
        eprintln!("{}", formatted);

        // Optionally write to file
        if self.config.log_to_file {
            if let Some(ref log_path) = self.config.log_file_path {
                let _ = self.write_to_file(log_path, &formatted);
            }
        }
    }

    /// Write a log message to file
    fn write_to_file(&self, path: &PathBuf, message: &str) -> std::io::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        // Strip ANSI color codes for file output
        let clean_message = strip_ansi_codes(message);
        writeln!(file, "{}", clean_message)?;
        Ok(())
    }
}

/// Initialize the global logger
pub fn init(config: LoggerConfig) {
    let logger = Logger::new(config);
    let mut global = LOGGER.lock().unwrap();
    *global = Some(logger);
}

/// Check if logger is initialized
pub fn is_initialized() -> bool {
    LOGGER.lock().unwrap().is_some()
}

/// Log a debug message
pub fn debug(message: &str) {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        logger.log(LogLevel::Debug, message);
    }
}

/// Log an info message
pub fn info(message: &str) {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        logger.log(LogLevel::Info, message);
    }
}

/// Log a warning message
pub fn warn(message: &str) {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        logger.log(LogLevel::Warn, message);
    }
}

/// Log an error message
pub fn error(message: &str) {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        logger.log(LogLevel::Error, message);
    }
}

/// Get the default log file path
pub fn default_log_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".nsfw").join("logs").join("nsfw.log")
    } else {
        PathBuf::from("nsfw.log")
    }
}

/// Strip ANSI color codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

/// Create a logger config from CLI flags and user config
pub fn create_config(
    verbose: bool,
    log_to_file: bool,
    disable_colors: bool,
) -> LoggerConfig {
    LoggerConfig {
        min_level: if verbose {
            LogLevel::Debug
        } else {
            LogLevel::Info
        },
        log_to_file,
        log_file_path: if log_to_file {
            Some(default_log_path())
        } else {
            None
        },
        disable_colors,
        include_timestamp: log_to_file, // Include timestamps when logging to file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert_eq!(config.min_level, LogLevel::Info);
        assert!(!config.log_to_file);
        assert!(!config.disable_colors);
    }

    #[test]
    fn test_create_config_verbose() {
        let config = create_config(true, false, false);
        assert_eq!(config.min_level, LogLevel::Debug);
        assert!(!config.log_to_file);
    }

    #[test]
    fn test_create_config_with_file() {
        let config = create_config(false, true, false);
        assert_eq!(config.min_level, LogLevel::Info);
        assert!(config.log_to_file);
        assert!(config.log_file_path.is_some());
        assert!(config.include_timestamp);
    }

    #[test]
    fn test_strip_ansi_codes() {
        let colored = "\x1b[31mRed Text\x1b[0m";
        let stripped = strip_ansi_codes(colored);
        assert_eq!(stripped, "Red Text");
    }

    #[test]
    fn test_default_log_path() {
        let path = default_log_path();
        assert!(path.to_string_lossy().contains(".nsfw"));
        assert!(path.to_string_lossy().contains("logs"));
        assert!(path.to_string_lossy().contains("nsfw.log"));
    }
}
