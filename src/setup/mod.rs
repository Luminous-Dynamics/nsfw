//! Setup wizard for NSFW
//! 
//! Handles first-time setup including:
//! - WSL2 detection and installation
//! - Linux distro detection and installation
//! - Nix installation and configuration

pub mod wsl_detector;
pub mod distro_installer;
pub mod nix_installer;
pub mod wsl_command;

use anyhow::Result;
use crate::ui::{OutputFormatter, MessageType};

pub struct SetupWizard {
    auto_yes: bool,
    interactive: bool,
}

impl SetupWizard {
    pub fn new(auto_yes: bool, interactive: bool) -> Self {
        Self { auto_yes, interactive }
    }

    pub fn run(&self) -> Result<()> {
        self.print_header();

        // Step 1: Check WSL2
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            "Checking system requirements..."
        ));
        eprintln!();

        let wsl_status = wsl_detector::check_wsl2()?;
        let distro_status = wsl_detector::check_linux_distro()?;
        let nix_status = if distro_status.is_installed {
            nix_installer::check_nix(&distro_status.distro_name)?
        } else {
            nix_installer::NixStatus::default()
        };

        // Show status
        self.print_status(&wsl_status, &distro_status, &nix_status);

        // Determine what needs to be done
        let needs_wsl = !wsl_status.is_installed;
        let needs_distro = !distro_status.is_installed;
        let needs_nix = !nix_status.is_installed;

        if !needs_wsl && !needs_distro && !needs_nix {
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Success,
                "✓ Your system is already configured! NSFW is ready to use."
            ));
            return Ok(());
        }

        // Ask for confirmation
        if !self.auto_yes {
            eprintln!();
            self.print_installation_plan(needs_wsl, needs_distro, needs_nix, &distro_status);
            
            if !self.confirm("Continue with automatic setup?")? {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Warning,
                    "Setup cancelled. Run 'nsfw setup' again when ready."
                ));
                return Ok(());
            }
        }

        // Execute setup steps
        if needs_wsl {
            self.install_wsl2()?;
        }

        if needs_distro {
            let distro = if self.interactive {
                self.choose_distro()?
            } else {
                "Ubuntu-24.04".to_string()
            };
            distro_installer::install_distro(&distro)?;
        }

        if needs_nix {
            nix_installer::install_nix(&distro_status.distro_name, self.auto_yes)?;
        }

        // Final success message
        eprintln!();
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Success,
            "🎉 Setup complete! NSFW is ready to use."
        ));
        eprintln!();
        eprintln!("Try running: nsfw search python");

        Ok(())
    }

    fn print_header(&self) {
        eprintln!("{}", OutputFormatter::format_section("NSFW Setup Wizard"));
    }

    fn print_status(
        &self,
        wsl: &wsl_detector::WSL2Status,
        distro: &wsl_detector::DistroStatus,
        nix: &nix_installer::NixStatus,
    ) {
        let wsl_icon = if wsl.is_installed { "✓" } else { "✗" };
        let distro_icon = if distro.is_installed { "✓" } else { "✗" };
        let nix_icon = if nix.is_installed { "✓" } else { "✗" };

        eprintln!("  {} WSL2: {}", wsl_icon, if wsl.is_installed { 
            format!("Enabled (version {})", wsl.version.as_deref().unwrap_or("unknown"))
        } else { 
            "Not installed".to_string() 
        });

        eprintln!("  {} Linux distribution: {}", distro_icon, if distro.is_installed {
            log::debug!("Detected distro name: '{}'", distro.distro_name);
            format!("{}", distro.distro_name)
        } else {
            "None found".to_string()
        });

        eprintln!("  {} Nix package manager: {}", nix_icon, if nix.is_installed {
            format!("Installed ({})", nix.version.as_deref().unwrap_or("unknown"))
        } else {
            "Not installed".to_string()
        });
    }

    fn print_installation_plan(
        &self,
        needs_wsl: bool,
        needs_distro: bool,
        needs_nix: bool,
        _distro_status: &wsl_detector::DistroStatus,
    ) {
        eprintln!("{}", OutputFormatter::format_section("Installation Plan"));
        
        if needs_wsl {
            eprintln!("  • Install WSL2 (Windows Subsystem for Linux)");
        }
        
        if needs_distro {
            eprintln!("  • Install Ubuntu 24.04 LTS");
        }
        
        if needs_nix {
            eprintln!("  • Install Nix package manager");
            eprintln!("  • Configure Nix daemon");
            eprintln!("  • Add user to nix-users group");
            eprintln!("  • Enable experimental features");
        }

        eprintln!();
        eprintln!("  Estimated time: 5-15 minutes");
    }

    fn confirm(&self, message: &str) -> Result<bool> {
        use std::io::{self, Write};
        
        eprint!("{} [Y/n]: ", message);
        io::stderr().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let answer = input.trim().to_lowercase();
        Ok(answer.is_empty() || answer == "y" || answer == "yes")
    }

    fn choose_distro(&self) -> Result<String> {
        use std::io::{self, Write};
        
        eprintln!("{}", OutputFormatter::format_section("Choose Linux Distribution"));
        eprintln!("  1. Ubuntu 24.04 LTS (recommended) ⭐");
        eprintln!("  2. Debian 12");
        eprintln!("  3. Ubuntu 22.04 LTS");
        eprintln!();
        eprint!("Select [1-3] (default: 1): ");
        io::stderr().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let choice = input.trim();
        let distro = match choice {
            "2" => "Debian",
            "3" => "Ubuntu-22.04",
            _ => "Ubuntu-24.04", // Default to Ubuntu 24.04
        };
        
        Ok(distro.to_string())
    }

    fn install_wsl2(&self) -> Result<()> {
        eprintln!();
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            "📦 Installing WSL2..."
        ));
        
        wsl_detector::install_wsl2()?;
        
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Success,
            "✓ WSL2 installed successfully"
        ));
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Warning,
            "⚠️  Please restart your computer, then run 'nsfw setup' again."
        ));
        
        std::process::exit(0);
    }
}
