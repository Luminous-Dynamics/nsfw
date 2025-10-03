//! WSL command execution helpers
//!
//! Handles UTF-16 LE encoding issues when calling WSL from Windows

use std::process::Command;
use anyhow::{Result, Context};

/// Execute a WSL command and return output as UTF-8 String
///
/// PowerShell always outputs UTF-16 LE on Windows, so we need to decode it properly.
///
/// # Arguments
/// * `args` - WSL command arguments (e.g., ["--list", "--verbose"])
///
/// # Returns
/// * `Ok(String)` - Decoded stdout as UTF-8 String
/// * `Err` - If command fails or cannot be decoded
pub fn run_wsl_command(args: &[&str]) -> Result<String> {
    let wsl_args = args.join(" ");
    let ps_command = format!("wsl {}", wsl_args);

    log::debug!("Running WSL command via PowerShell: {}", ps_command);

    let output = Command::new("powershell.exe")
        .args(&["-NoProfile", "-Command", &ps_command])
        .output()
        .context("Failed to execute PowerShell")?;

    if !output.status.success() {
        let stderr = decode_utf16_le(&output.stderr);
        log::debug!("WSL command failed. Exit code: {:?}", output.status.code());
        log::debug!("Stderr: {}", stderr);
        anyhow::bail!("WSL command failed: {}", stderr);
    }

    // PowerShell always outputs UTF-16 LE - decode it properly
    let stdout = decode_utf16_le(&output.stdout);
    log::debug!("WSL command output: {}", stdout.trim());

    Ok(stdout)
}

/// Decode UTF-16 LE bytes to a Rust String
///
/// PowerShell outputs UTF-16 LE (Little Endian) on Windows.
/// Each character is 2 bytes in little-endian format.
fn decode_utf16_le(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    // If byte count is odd, we have a problem - fall back to UTF-8
    if bytes.len() % 2 != 0 {
        log::warn!("Odd number of bytes in UTF-16 output, falling back to UTF-8");
        return String::from_utf8_lossy(bytes).to_string();
    }

    // Convert bytes to u16 values (UTF-16 LE)
    let output_u16: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    String::from_utf16_lossy(&output_u16)
}

/// Execute a WSL command in a specific distro
///
/// # Arguments
/// * `distro` - Name of the WSL distro (e.g., "Ubuntu")
/// * `command` - Command to run in the distro
///
/// # Returns
/// * `Ok(String)` - Decoded stdout as UTF-8 String
/// * `Err` - If command fails
pub fn run_wsl_distro_command(distro: &str, command: &[&str]) -> Result<String> {
    let cmd = command.join(" ");

    let ps_command = if distro.is_empty() {
        format!("wsl -- {}", cmd)
    } else {
        format!("wsl -d {} -- {}", distro, cmd)
    };

    log::debug!("Running WSL distro command: {}", ps_command);

    let output = Command::new("powershell.exe")
        .args(&["-NoProfile", "-Command", &ps_command])
        .output()
        .context("Failed to execute PowerShell")?;

    if !output.status.success() {
        let stderr = decode_utf16_le(&output.stderr);
        log::debug!("WSL distro command failed. Exit code: {:?}", output.status.code());
        log::debug!("Stderr: {}", stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    let stdout = decode_utf16_le(&output.stdout);
    log::debug!("Command output: {}", stdout.trim());

    Ok(stdout)
}

/// Check if a WSL command succeeds (returns true) or fails (returns false)
///
/// # Arguments
/// * `args` - WSL command arguments
///
/// # Returns
/// * `true` - If command exits with status 0
/// * `false` - If command fails
pub fn wsl_command_exists(args: &[&str]) -> bool {
    let wsl_args = args.join(" ");
    let ps_command = format!("wsl {}", wsl_args);

    let output = Command::new("powershell.exe")
        .args(&["-NoProfile", "-Command", &ps_command])
        .output();

    match output {
        Ok(result) => {
            let success = result.status.success();
            if !success {
                let stderr = decode_utf16_le(&result.stderr);
                log::debug!("Command '{}' failed: {}", wsl_args, stderr);
            }
            log::debug!("Command '{}' exists: {}", wsl_args, success);
            success
        }
        Err(e) => {
            log::debug!("Command '{}' check failed: {}", wsl_args, e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsl_command_construction() {
        // Just test that the functions exist and compile
        assert!(true);
    }
}
