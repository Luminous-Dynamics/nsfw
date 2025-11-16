/// Output formatting utilities
use colored::*;
use crate::nix_ops::types::{SearchResult, InstalledPackage, PackageInfo};

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

        // Add a newline before results
        output.push('\n');

        for (i, result) in results.iter().enumerate() {
            // Add separator between results (except before first one)
            if i > 0 {
                output.push_str(&format!("{}\n", "─".repeat(70).bright_black()));
            }

            // Package number and name on same line
            if show_numbers {
                let number = format!("{:>2}.", i + 1).bright_cyan().bold();
                let name = result.pname.bright_green().bold();
                output.push_str(&format!("{} {}", number, name));

                // Add version on same line with separator
                let version = result.version.bright_yellow();
                output.push_str(&format!("  {}\n", version));
            } else {
                // Without numbers, just show name and version
                let name = result.pname.bright_green().bold();
                let version = result.version.bright_yellow();
                output.push_str(&format!("{} {}\n", name, version));
            }

            // Description with smart truncation
            if !result.description.is_empty() {
                let desc = Self::truncate_description(&result.description, 100);
                let indentation = if show_numbers { "    " } else { "   " };
                output.push_str(&format!("{}{}\n", indentation, desc.bright_white()));
            }

            output.push('\n');
        }

        output
    }

    /// Truncate description intelligently with ellipsis
    fn truncate_description(text: &str, max_len: usize) -> String {
        // Remove excess whitespace first
        let cleaned: String = text.split_whitespace().collect::<Vec<_>>().join(" ");

        if cleaned.len() <= max_len {
            return cleaned;
        }

        // Find a good break point (space near the limit)
        let truncate_at = cleaned[..max_len]
            .rfind(' ')
            .unwrap_or(max_len);

        format!("{}...", &cleaned[..truncate_at])
    }

    /// Format installed packages for display
    pub fn format_installed_packages(packages: &[InstalledPackage], detailed: bool) -> String {
        if packages.is_empty() {
            return Self::format_message(MessageType::Info, "No packages installed");
        }

        let mut output = String::new();

        // Add a newline before results
        output.push('\n');

        if detailed {
            // Detailed view with separators
            for (i, pkg) in packages.iter().enumerate() {
                // Add separator between packages (except before first one)
                if i > 0 {
                    output.push_str(&format!("{}\n", "─".repeat(70).bright_black()));
                }

                // Package number and name on same line
                let number = format!("{:>2}.", i + 1).bright_cyan().bold();
                let name = pkg.name.bright_green().bold();
                let version = pkg.version.bright_yellow();
                output.push_str(&format!("{} {}  {}\n", number, name, version));

                // Store path indented
                output.push_str(&format!("    {}: {}\n",
                    "Store path".bright_black(),
                    pkg.store_path.bright_black()));

                output.push('\n');
            }
        } else {
            // Compact view - name and version on same line
            for (i, pkg) in packages.iter().enumerate() {
                let number = format!("{:>2}.", i + 1).bright_cyan().bold();
                let name = pkg.name.bright_green().bold();
                let version = pkg.version.bright_yellow();
                output.push_str(&format!("{} {}  {}\n", number, name, version));
            }
            output.push('\n');
        }

        output
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

    /// Format detailed package information
    pub fn format_package_info(info: &PackageInfo) -> String {
        let mut output = String::new();

        // Package name and version
        output.push_str(&format!("{} {}\n",
            info.pname.bright_green().bold(),
            format!("v{}", info.version).yellow()
        ));
        output.push_str(&format!("{}\n\n", "─".repeat(60).bright_black()));

        // Description
        if !info.description.is_empty() {
            output.push_str(&format!("{}: {}\n\n",
                "Description".bright_black(),
                info.description.bright_white()
            ));
        }

        // Homepage
        if let Some(homepage) = &info.homepage {
            output.push_str(&format!("{}: {}\n",
                "Homepage".bright_black(),
                homepage.cyan()
            ));
        }

        // License
        if let Some(license) = &info.license {
            output.push_str(&format!("{}: {}\n",
                "License".bright_black(),
                license.bright_white()
            ));
        }

        // Outputs
        if !info.outputs.is_empty() {
            output.push_str(&format!("{}: {}\n",
                "Outputs".bright_black(),
                info.outputs.join(", ").bright_white()
            ));
        }

        // Maintainers
        if !info.maintainers.is_empty() {
            output.push_str(&format!("{}: {}\n",
                "Maintainers".bright_black(),
                info.maintainers.join(", ").bright_white()
            ));
        }

        // Platforms
        if !info.platforms.is_empty() {
            output.push_str(&format!("{}: {}\n",
                "Platforms".bright_black(),
                info.platforms.join(", ").bright_white()
            ));
        }

        output
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
    fn test_truncate_description() {
        let text = "This is a very long description that should be truncated at some point";
        let truncated = OutputFormatter::truncate_description(text, 30);
        assert!(truncated.contains("..."));
        assert!(truncated.len() < text.len());

        // Test short text (shouldn't be truncated)
        let short_text = "Short text";
        let not_truncated = OutputFormatter::truncate_description(short_text, 30);
        assert!(!not_truncated.contains("..."));
        assert_eq!(not_truncated, short_text);
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
