//! Nix installation and configuration for WSL2

use anyhow::{Result, Context, bail};
use std::process::Command;
use crate::ui::{OutputFormatter, MessageType, ProgressIndicator};
use super::wsl_command;

#[derive(Debug, Default)]
pub struct NixStatus {
    pub is_installed: bool,
    pub version: Option<String>,
    pub daemon_running: bool,
    pub in_nix_users: bool,
    pub experimental_enabled: bool,
}

pub fn check_nix(distro: &str) -> Result<NixStatus> {
    // Check if 'nix' command exists in WSL
    // Use PowerShell wrapper to avoid UTF-16 encoding issues
    log::debug!("Checking for Nix in distro: '{}'", distro);

    let is_installed = if !distro.is_empty() {
        wsl_command::wsl_command_exists(&["-d", distro, "--", "which", "nix"])
    } else {
        wsl_command::wsl_command_exists(&["--", "which", "nix"])
    };

    log::debug!("Nix installed: {}", is_installed);

    if !is_installed {
        return Ok(NixStatus::default());
    }

    // Get version
    let version = get_nix_version(distro)?;

    // Check daemon
    let daemon_running = check_daemon_running(distro)?;

    // Check user group
    let in_nix_users = check_nix_users_group(distro)?;

    // Check experimental features
    let experimental_enabled = check_experimental_features(distro)?;

    Ok(NixStatus {
        is_installed: true,
        version: Some(version),
        daemon_running,
        in_nix_users,
        experimental_enabled,
    })
}

pub fn install_nix(distro: &str, auto_yes: bool) -> Result<()> {
    eprintln!();
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "📦 Installing Nix package manager..."
    ));

    let progress = ProgressIndicator::spinner("Downloading Nix installer...");

    // Use the Determinate Systems Nix installer (recommended for WSL2)
    let install_cmd = format!(
        "curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install linux --init none {}",
        if auto_yes { "--no-confirm" } else { "" }
    );

    progress.set_message("Installing Nix...");

    let output = Command::new("wsl")
        .args(&["-d", distro, "--", "bash", "-c", &install_cmd])
        .output()
        .context("Failed to install Nix")?;

    progress.finish_and_clear();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        log::error!("Nix installation command failed");
        log::error!("Command: {}", install_cmd);
        log::error!("Exit code: {:?}", output.status.code());
        log::error!("Stdout: {}", stdout);
        log::error!("Stderr: {}", stderr);

        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Error,
            &format!("Nix installation failed. Run with --verbose for details.")
        ));

        if log::log_enabled!(log::Level::Debug) {
            eprintln!("Error output: {}", stderr);
        }

        bail!("Nix installation failed");
    }

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Success,
        "✓ Nix installed successfully"
    ));

    // Configure Nix
    configure_nix(distro)?;

    Ok(())
}

fn configure_nix(distro: &str) -> Result<()> {
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "⚙️  Configuring Nix..."
    ));

    // Create nix.conf if it doesn't exist
    let config_cmd = r#"
mkdir -p ~/.config/nix
if ! grep -q 'experimental-features' ~/.config/nix/nix.conf 2>/dev/null; then
    echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf
fi
"#;

    Command::new("wsl")
        .args(&["-d", distro, "--", "bash", "-c", config_cmd])
        .output()
        .context("Failed to configure Nix")?;

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Success,
        "✓ Nix configured with experimental features"
    ));

    // Add user to nix-users group if not already
    add_to_nix_users(distro)?;

    // Configure channels
    configure_channels(distro)?;

    Ok(())
}

fn add_to_nix_users(distro: &str) -> Result<()> {
    let groups = wsl_command::run_wsl_distro_command(distro, &["groups"])?;

    if !groups.contains("nix-users") {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            "Adding user to nix-users group..."
        ));

        let username = wsl_command::run_wsl_distro_command(distro, &["whoami"])?;
        let username = username.trim();

        // Use bash -c to run the sudo command
        wsl_command::run_wsl_distro_command(
            distro,
            &["bash", "-c", &format!("sudo usermod -a -G nix-users {}", username)]
        ).context("Failed to add user to nix-users group")?;

        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Success,
            "✓ Added to nix-users group"
        ));
    }

    Ok(())
}

fn configure_channels(distro: &str) -> Result<()> {
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "Configuring Nix channels..."
    ));

    // Add nixpkgs-unstable channel
    Command::new("wsl")
        .args(&["-d", distro, "--", "nix-channel", "--add", "https://nixos.org/channels/nixpkgs-unstable", "nixpkgs"])
        .output()?;

    // Update channels (this can take a while)
    let progress = ProgressIndicator::spinner("Updating channels...");

    let update_result = Command::new("wsl")
        .args(&["-d", distro, "--", "nix-channel", "--update"])
        .output()?;

    progress.finish_and_clear();

    if update_result.status.success() {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Success,
            "✓ Channels configured and updated"
        ));
    }

    Ok(())
}

fn get_nix_version(distro: &str) -> Result<String> {
    let version = wsl_command::run_wsl_distro_command(distro, &["nix", "--version"])?;
    Ok(version.trim().to_string())
}

fn check_daemon_running(distro: &str) -> Result<bool> {
    Ok(wsl_command::wsl_command_exists(&["-d", distro, "--", "pgrep", "nix-daemon"]))
}

fn check_nix_users_group(distro: &str) -> Result<bool> {
    let groups = wsl_command::run_wsl_distro_command(distro, &["groups"])?;
    Ok(groups.contains("nix-users"))
}

fn check_experimental_features(distro: &str) -> Result<bool> {
    match wsl_command::run_wsl_distro_command(distro, &["cat", "~/.config/nix/nix.conf"]) {
        Ok(config) => Ok(config.contains("experimental-features")),
        Err(_) => Ok(false), // File doesn't exist or can't be read
    }
}
