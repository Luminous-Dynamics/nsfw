use std::time::Instant;
use colored::Colorize;

/// Simple performance timer for operations
pub struct PerformanceTimer {
    start: Instant,
    operation: String,
}

impl PerformanceTimer {
    /// Start timing an operation
    pub fn start(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    /// Stop timing and return elapsed time in seconds
    pub fn elapsed(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Stop timing and print elapsed time
    pub fn finish(&self) {
        let elapsed = self.elapsed();
        let formatted = format_duration(elapsed);

        crate::logging::debug(&format!(
            "⏱️  {} completed in {}",
            self.operation,
            formatted
        ));
    }

    /// Stop timing and print elapsed time if verbose
    pub fn finish_with_message(&self, message: &str) {
        let elapsed = self.elapsed();
        let formatted = format_duration(elapsed);

        crate::logging::info(&format!("{} (took {})", message, formatted.dimmed()));
    }
}

/// Format duration in a human-readable way
fn format_duration(seconds: f64) -> String {
    if seconds < 0.001 {
        format!("{:.2}µs", seconds * 1_000_000.0)
    } else if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.2}s", seconds)
    } else {
        let mins = (seconds / 60.0).floor() as u32;
        let secs = seconds % 60.0;
        format!("{}m {:.1}s", mins, secs)
    }
}

/// Global performance statistics
#[derive(Debug, Default)]
pub struct PerformanceStats {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub packages_installed: usize,
    pub packages_removed: usize,
}

impl PerformanceStats {
    /// Get cache hit rate as a percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }

    /// Print performance statistics
    pub fn print(&self) {
        if self.cache_hits > 0 || self.cache_misses > 0 {
            crate::logging::info(&format!(
                "Cache performance: {} hits, {} misses ({:.1}% hit rate)",
                self.cache_hits,
                self.cache_misses,
                self.cache_hit_rate()
            ));
        }

        if self.packages_installed > 0 {
            crate::logging::info(&format!("Packages installed: {}", self.packages_installed));
        }

        if self.packages_removed > 0 {
            crate::logging::info(&format!("Packages removed: {}", self.packages_removed));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_microseconds() {
        let result = format_duration(0.0000005);
        assert!(result.contains("µs"));
    }

    #[test]
    fn test_format_duration_milliseconds() {
        let result = format_duration(0.123);
        assert_eq!(result, "123ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        let result = format_duration(5.67);
        assert_eq!(result, "5.67s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let result = format_duration(125.5);
        assert_eq!(result, "2m 5.5s");
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut stats = PerformanceStats::default();
        stats.cache_hits = 75;
        stats.cache_misses = 25;

        assert_eq!(stats.cache_hit_rate(), 75.0);
    }

    #[test]
    fn test_cache_hit_rate_zero_total() {
        let stats = PerformanceStats::default();
        assert_eq!(stats.cache_hit_rate(), 0.0);
    }
}
