/// Progress indicator utilities
use indicatif::ProgressBar;
use std::time::Duration;

/// Progress indicator for different operation types
pub enum ProgressIndicator {
    Spinner(ProgressBar),
    Bar(ProgressBar),
}

impl ProgressIndicator {
    /// Create a new spinner-style progress indicator
    pub fn spinner(message: &str) -> Self {
        Self::Spinner(super::create_spinner(message))
    }

    /// Create a new progress bar with known length
    pub fn bar(len: u64, message: &str) -> Self {
        Self::Bar(super::create_progress_bar(len, message))
    }

    /// Update the message
    pub fn set_message(&self, message: &str) {
        match self {
            Self::Spinner(pb) | Self::Bar(pb) => pb.set_message(message.to_string()),
        }
    }

    /// Increment progress (for bar)
    pub fn inc(&self, delta: u64) {
        if let Self::Bar(pb) = self {
            pb.inc(delta);
        }
    }

    /// Set position (for bar)
    pub fn set_position(&self, pos: u64) {
        if let Self::Bar(pb) = self {
            pb.set_position(pos);
        }
    }

    /// Finish with success message
    pub fn finish_with_message(&self, message: &str) {
        match self {
            Self::Spinner(pb) | Self::Bar(pb) => {
                pb.finish_with_message(message.to_string());
            }
        }
    }

    /// Finish and clear
    pub fn finish_and_clear(&self) {
        match self {
            Self::Spinner(pb) | Self::Bar(pb) => {
                pb.finish_and_clear();
            }
        }
    }

    /// Tick the spinner
    pub fn tick(&self) {
        if let Self::Spinner(pb) = self {
            pb.tick();
        }
    }

    /// Enable steady tick for spinner
    pub fn enable_steady_tick(&self, interval: Duration) {
        if let Self::Spinner(pb) = self {
            pb.enable_steady_tick(interval);
        }
    }
}

impl Drop for ProgressIndicator {
    fn drop(&mut self) {
        self.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let progress = ProgressIndicator::spinner("Testing");
        progress.set_message("Updated");
        progress.finish_and_clear();
    }

    #[test]
    fn test_bar_creation() {
        let progress = ProgressIndicator::bar(100, "Processing");
        progress.inc(10);
        progress.set_position(50);
        progress.finish_with_message("Done");
    }
}
