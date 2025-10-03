//! Linux distribution installer for WSL2

use anyhow::{Result, Context, bail};
use std::process::Command;
use crate::ui::{OutputFormatter, MessageType};

pub fn install_distro(distro: &str) -> Result<()> {
    eprintln!();
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        &format!("📦 Installing {}...", distro)
    ));

    // Use 'wsl --install -d <distro>' to install specific distribution
    let output = Command::new("wsl")
        .args(&["--install", "-d", distro])
        .output()
        .context("Failed to install Linux distribution")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Check if distro is already installed
        if stderr.contains("already installed") {
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Info,
                &format!("ℹ {} is already installed", distro)
            ));
            return Ok(());
        }
        
        bail!("Failed to install {}: {}", distro, stderr);
    }

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Success,
        &format!("✓ {} installed successfully", distro)
    ));

    Ok(())
}
