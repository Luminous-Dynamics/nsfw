/// Output formatting utilities
use colored::*;
use crate::nix_ops::types::{SearchResult, InstalledPackage};

/// Message type for colored output
pub enum MessageType {
    Success,
    Error,
    Warning,
    Info,
    Question,
}

/// Output formatter for consistent, colorful CLI output
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format a message with appropriate color and icon
    pub fn format_message(msg_type: MessageType, message: &str) -> String {
        match msg_type {
            MessageType::Success => format!("{} {}", "✓".green().bold(), message.bright_white()),
            MessageType::Error => format!("{} {}", "✗".red().bold(), message.bright_white()),
            MessageType::Warning => format!("{} {}", "⚠".yellow().bold(), message.bright_white()),
            MessageType::Info => format!("{} {}", "ℹ".cyan().bold(), message.bright_white()),
            MessageType::Question => format!("{} {}", "?".magenta().bold(), message.bright_white()),
        }
    }

    /// Format search results for display
    pub fn format_search_results(results: &[SearchResult], show_numbers: bool) -> String {
        if results.is_empty() {
            return Self::format_message(MessageType::Warning, "No results found");
        }

        let mut output = String::new();

        for (i, result) in results.iter().enumerate() {
            if show_numbers {
                output.push_str(&format!("{}. ", (i + 1).to_string().cyan().bold()));
            }

            // Package name in bright green
            output.push_str(&result.pname.bright_green().bold().to_string());
            output.push('\n');

            // Version in yellow
            output.push_str(&format!("   {}: {}\n", "Version".bright_black(), result.version.yellow()));

            // Description (wrapped if too long)
            if !result.description.is_empty() {
                let desc = Self::wrap_text(&result.description, 70);
                output.push_str(&format!("   {}: {}\n", "Description".bright_black(), desc.bright_white()));
            }

            output.push('\n');
        }

        output
    }

    /// Format installed packages for display
    pub fn format_installed_packages(packages: &[InstalledPackage], detailed: bool) -> String {
        if packages.is_empty() {
            return Self::format_message(MessageType::Info, "No packages installed");
        }

        let mut output = String::new();

        for (i, pkg) in packages.iter().enumerate() {
            output.push_str(&format!("{}. ", (i + 1).to_string().cyan().bold()));
            output.push_str(&pkg.name.bright_green().bold().to_string());

            if detailed {
                output.push('\n');
                output.push_str(&format!("   {}: {}\n", "Version".bright_black(), pkg.version.yellow()));
                output.push_str(&format!("   {}: {}\n", "Store path".bright_black(), pkg.store_path.bright_black()));
            }

            output.push('\n');
        }

        output
    }

    /// Wrap text to specified width
    fn wrap_text(text: &str, width: usize) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in words {
            if current_line.len() + word.len() + 1 <= width {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines.join("\n   ")
    }

    /// Format an error with suggestions
    pub fn format_error_with_suggestion(error: &str, suggestion: &str) -> String {
        format!(
            "{}\n\n{} {}",
            Self::format_message(MessageType::Error, error),
            "Suggestion:".bright_cyan().bold(),
            suggestion.bright_white()
        )
    }

    /// Format a section header
    pub fn format_section(title: &str) -> String {
        format!("\n{}\n{}", title.bright_cyan().bold(), "─".repeat(title.len()).bright_black())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message() {
        let msg = OutputFormatter::format_message(MessageType::Success, "Operation complete");
        assert!(msg.contains("Operation complete"));
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a very long description that should be wrapped";
        let wrapped = OutputFormatter::wrap_text(text, 20);
        assert!(wrapped.contains('\n'));
    }

    #[test]
    fn test_format_search_results_empty() {
        let results: Vec<SearchResult> = vec![];
        let output = OutputFormatter::format_search_results(&results, true);
        assert!(output.contains("No results"));
    }

    #[test]
    fn test_format_installed_packages_empty() {
        let packages: Vec<InstalledPackage> = vec![];
        let output = OutputFormatter::format_installed_packages(&packages, false);
        assert!(output.contains("No packages"));
    }
}
