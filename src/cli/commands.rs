use anyhow::Result;
use std::path::PathBuf;

use crate::nix_ops::{BridgedNixExecutor, NixError};
use crate::templates::{WrapperGenerator, PackageInfo, WrapperType};
use crate::wsl2::RealWSL2Bridge;
use crate::cache::SearchCache;
use crate::ui::{ProgressIndicator, OutputFormatter, MessageType};

pub fn search(query: &str, limit: usize, format: &str) -> Result<()> {
    // Show search header
    eprintln!("{}", OutputFormatter::format_section(&format!("Searching for '{}'", query)));

    // Check cache first
    if let Some(cached_results) = SearchCache::get(query, limit) {
        log::debug!("Using cached results for '{}'", query);

        if format == "json" {
            let json = serde_json::to_string_pretty(&cached_results)?;
            println!("{}", json);
        } else {
            eprintln!("{}", OutputFormatter::format_message(MessageType::Info, "Using cached results"));
            print!("{}", OutputFormatter::format_search_results(&cached_results, true));
        }

        return Ok(());
    }

    // Create progress spinner
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");

    // Create bridged executor that uses WSL2
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    let is_first_time = !executor.is_cache_built();

    match executor.check_nix_available() {
        Ok(version) => {
            progress.finish_and_clear();
            if log::log_enabled!(log::Level::Debug) {
                eprintln!("{}", OutputFormatter::format_message(MessageType::Info, &format!("Using: {}", version)));
            }

            // Inform user about first-time delay only if cache doesn't exist
            if is_first_time {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Warning,
                    "⏳ First-time setup: Downloading package database (200-500MB, 2-10 min)"
                ));
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Info,
                    "   This is a ONE-TIME operation. All future searches will be instant!"
                ));
            }
        }
        Err(e) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(MessageType::Error, &e.to_string()));
            return Err(e.into());
        }
    }

    // Perform search with appropriate message
    let search_msg = if is_first_time {
        format!("Downloading database and searching for '{}'... (please wait 2-10 min)", query)
    } else {
        format!("Searching nixpkgs for '{}'...", query)
    };
    progress.set_message(&search_msg);
    match executor.search(query, limit) {
        Ok(results) => {
            progress.finish_and_clear();

            // Cache the results
            SearchCache::put(query, limit, results.clone());

            // Output results based on format
            match format {
                "json" => {
                    let json = serde_json::to_string_pretty(&results)?;
                    println!("{}", json);
                }
                _ => {
                    if results.is_empty() {
                        eprintln!("{}", OutputFormatter::format_message(MessageType::Warning, &format!("No results found for '{}'", query)));
                    } else {
                        eprintln!("{}", OutputFormatter::format_message(MessageType::Success, &format!("Found {} result(s)", results.len())));
                        print!("{}", OutputFormatter::format_search_results(&results, true));
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(MessageType::Error, &format!("Search failed: {}", e)));
            Err(e.into())
        }
    }
}

pub fn install(package: &str, yes: bool) -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section(&format!("Installing '{}'", package)));

    // Create bridged executor that uses WSL2
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprintln!("{}", OutputFormatter::format_message(MessageType::Error, &e.to_string()));
        return Err(e.into());
    }
    progress.finish_and_clear();

    // Confirm unless --yes flag
    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Proceed with installation of '{}'?", package))
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("{}", OutputFormatter::format_message(MessageType::Info, "Installation cancelled"));
            return Ok(());
        }
    }

    // Perform installation with progress indicator
    let progress = ProgressIndicator::spinner(&format!("Installing '{}'...", package));
    match executor.install(package) {
        Ok(()) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(MessageType::Success, &format!("Successfully installed '{}'", package)));
            Ok(())
        }
        Err(NixError::AlreadyInstalled(_)) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(MessageType::Info, &format!("Package '{}' is already installed", package)));
            Ok(())
        }
        Err(e) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_error_with_suggestion(
                &format!("Installation failed: {}", e),
                "Try updating your Nix channels with 'nsfw update' or check package name"
            ));
            Err(e.into())
        }
    }
}

pub fn remove(package: &str, yes: bool) -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section(&format!("Removing '{}'", package)));

    // Create bridged executor that uses WSL2
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprintln!("{}", OutputFormatter::format_message(MessageType::Error, &e.to_string()));
        return Err(e.into());
    }
    progress.finish_and_clear();

    // Confirm unless --yes flag
    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Proceed with removal of '{}'?", package))
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("{}", OutputFormatter::format_message(MessageType::Info, "Removal cancelled"));
            return Ok(());
        }
    }

    // Perform removal with progress indicator
    let progress = ProgressIndicator::spinner(&format!("Removing '{}'...", package));
    match executor.remove(package) {
        Ok(()) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(MessageType::Success, &format!("Successfully removed '{}'", package)));
            Ok(())
        }
        Err(NixError::NotInstalled(_)) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(MessageType::Warning, &format!("Package '{}' is not installed", package)));
            Ok(())
        }
        Err(e) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_error_with_suggestion(
                &format!("Removal failed: {}", e),
                "Check if the package name is correct with 'nsfw list'"
            ));
            Err(e.into())
        }
    }
}

pub fn list(detailed: bool, format: &str) -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section("Installed Packages"));

    // Create bridged executor that uses WSL2
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprintln!("{}", OutputFormatter::format_message(MessageType::Error, &e.to_string()));
        return Err(e.into());
    }

    // Get list of installed packages
    progress.set_message("Retrieving package list...");
    match executor.list() {
        Ok(packages) => {
            progress.finish_and_clear();

            // Output based on format
            match format {
                "json" => {
                    let json = serde_json::to_string_pretty(&packages)?;
                    println!("{}", json);
                }
                _ => {
                    if packages.is_empty() {
                        eprintln!("{}", OutputFormatter::format_message(MessageType::Info, "No packages installed"));
                    } else {
                        eprintln!("{}", OutputFormatter::format_message(MessageType::Success, &format!("{} package(s) installed", packages.len())));
                        print!("{}", OutputFormatter::format_installed_packages(&packages, detailed));
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_error_with_suggestion(
                &format!("Failed to list packages: {}", e),
                "Ensure Nix profile is initialized"
            ));
            Err(e.into())
        }
    }
}

pub fn info(package: &str) -> Result<()> {
    println!("ℹ️  Package info for '{}'", package);
    println!("⚠️  Info not yet implemented (Phase 1 Day 3-4)");
    Ok(())
}

pub fn update() -> Result<()> {
    println!("🔄 Updating package database");
    println!("⚠️  Update not yet implemented (Phase 1 Day 3-4)");
    Ok(())
}

pub fn generate_wrapper(package: &str, package_path: &str) -> Result<()> {
    println!("🔧 Generating wrapper for '{}'...", package);

    // Determine output directory (current directory by default)
    let output_dir = PathBuf::from(".");
    let generator = WrapperGenerator::new(output_dir);

    // Validate the Nix store path
    println!("   Validating Nix store path...");
    generator.validate_store_path(package_path)?;

    // Detect wrapper type based on package name
    let wrapper_type = generator.detect_wrapper_type(package);
    let wrapper_type_str = match wrapper_type {
        WrapperType::Console => "Console",
        WrapperType::Gui => "GUI",
        WrapperType::Vbs => "VBS (Silent)",
    };
    println!("   Detected type: {}", wrapper_type_str);

    // Create package info
    let package_info = PackageInfo::new(
        package.to_string(),
        package_path.to_string(),
        wrapper_type,
    );

    // Generate the wrapper
    println!("   Generating wrapper script...");
    let wrapper_path = generator.generate(&package_info)?;

    println!("✅ Wrapper generated successfully!");
    println!("   Location: {}", wrapper_path.display());
    println!("   You can now run: {}", wrapper_path.display());

    Ok(())
}
