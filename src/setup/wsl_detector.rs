//! WSL2 detection and installation

use anyhow::{Result, Context, bail};
use std::process::Command;
use super::wsl_command;

#[derive(Debug, Default)]
pub struct WSL2Status {
    pub is_installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Default)]
pub struct DistroStatus {
    pub is_installed: bool,
    pub distro_name: String,
}

pub fn check_wsl2() -> Result<WSL2Status> {
    // Try to run 'wsl --status'
    let output = Command::new("wsl")
        .args(&["--status"])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            Ok(WSL2Status {
                is_installed: true,
                version: Some(extract_version(&stdout)),
            })
        }
        _ => {
            // WSL might be installed but not configured
            // Try 'wsl --version' as fallback
            let version_output = Command::new("wsl")
                .args(&["--version"])
                .output();

            match version_output {
                Ok(result) if result.status.success() => {
                    Ok(WSL2Status {
                        is_installed: true,
                        version: Some("2.0+".to_string()),
                    })
                }
                _ => {
                    Ok(WSL2Status {
                        is_installed: false,
                        version: None,
                    })
                }
            }
        }
    }
}

pub fn check_linux_distro() -> Result<DistroStatus> {
    // Run 'wsl --list --verbose' to see installed distros
    // Use PowerShell wrapper to avoid UTF-16 LE encoding issues
    match wsl_command::run_wsl_command(&["--list", "--verbose"]) {
        Ok(stdout) => {
            log::debug!("WSL list output: {}", stdout);
            let distros = parse_distro_list(&stdout);

            if distros.is_empty() {
                log::debug!("No distros found in output");
                Ok(DistroStatus {
                    is_installed: false,
                    distro_name: String::new(),
                })
            } else {
                log::debug!("Found distros: {:?}", distros);
                // Return the first (default) distro
                Ok(DistroStatus {
                    is_installed: true,
                    distro_name: distros[0].clone(),
                })
            }
        }
        Err(e) => {
            log::debug!("Failed to list WSL distros: {}", e);
            Ok(DistroStatus {
                is_installed: false,
                distro_name: String::new(),
            })
        }
    }
}

pub fn install_wsl2() -> Result<()> {
    // Run 'wsl --install' which installs WSL2 and Ubuntu by default
    let output = Command::new("wsl")
        .args(&["--install"])
        .output()
        .context("Failed to run 'wsl --install'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("WSL installation failed: {}", stderr);
    }

    Ok(())
}

fn extract_version(wsl_status: &str) -> String {
    // Try to extract version from WSL status output
    for line in wsl_status.lines() {
        if line.contains("WSL version") || line.contains("Kernel version") {
            return line.split(':').nth(1)
                .unwrap_or("2.0+")
                .trim()
                .to_string();
        }
    }
    "2.0+".to_string()
}

fn parse_distro_list(output: &str) -> Vec<String> {
    let mut distros = Vec::new();

    // WSL output may contain UTF-16 BOM and special characters
    let cleaned = output.replace('\u{feff}', "").replace('\r', "");

    for line in cleaned.lines().skip(1) { // Skip header
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split by whitespace and filter out empty parts
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        // First part is either "*" or distro name, or "* distro"
        let mut name = parts[0];
        let mut idx = 0;

        // If first part is just "*", the name is in the next part
        if name == "*" && parts.len() > 1 {
            name = parts[1];
            idx = 1;
        } else {
            // Remove leading "*" if present
            name = name.trim_start_matches('*').trim();
        }

        // Skip if it's the header or empty
        if !name.is_empty() && name != "NAME" && parts.len() > idx + 1 {
            distros.push(name.to_string());
        }
    }

    distros
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_distro_list() {
        let output = "  NAME      STATE           VERSION\n* Ubuntu    Running         2\n  Debian    Stopped         2";
        let distros = parse_distro_list(output);
        assert_eq!(distros, vec!["Ubuntu", "Debian"]);
    }
}
