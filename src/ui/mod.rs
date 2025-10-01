/// UI utilities for enhanced user experience
///
/// Provides colored output, progress indicators, and interactive prompts
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::time::Duration;

pub mod progress;
pub mod output;

pub use progress::ProgressIndicator;
pub use output::{OutputFormatter, MessageType};

/// Create a spinner for long-running operations
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_draw_target(ProgressDrawTarget::stderr());
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .expect("Invalid progress template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a progress bar for operations with known length
pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_draw_target(ProgressDrawTarget::stderr());
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n[{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .expect("Invalid progress template")
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Print success message with checkmark
pub fn success(message: &str) {
    eprintln!("{} {}", "✓".green().bold(), message.bright_white());
}

/// Print error message with X
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message.bright_white());
}

/// Print warning message with triangle
pub fn warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow().bold(), message.bright_white());
}

/// Print info message with i
pub fn info(message: &str) {
    eprintln!("{} {}", "ℹ".cyan().bold(), message.bright_white());
}

/// Print a section header
pub fn section(title: &str) {
    eprintln!("\n{}", title.bright_cyan().bold());
    eprintln!("{}", "─".repeat(title.len()).bright_black());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_spinner() {
        let spinner = create_spinner("Testing...");
        assert!(true); // Just verify it doesn't panic
        spinner.finish_and_clear();
    }

    #[test]
    fn test_create_progress_bar() {
        let pb = create_progress_bar(100, "Processing");
        assert!(true); // Just verify it doesn't panic
        pb.finish_and_clear();
    }
}
