use anyhow::Result;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::nix_ops::{NixExecutor, BridgedNixExecutor, NixError};
use crate::templates::{WrapperGenerator, PackageInfo, WrapperType};
use crate::wsl2::RealWSL2Bridge;

pub fn search(query: &str, limit: usize, format: &str) -> Result<()> {
    println!("🔍 Searching for '{}'...", query);

    // Create bridged executor that uses WSL2
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    match executor.check_nix_available() {
        Ok(version) => {
            if log::log_enabled!(log::Level::Debug) {
                println!("   Using: {}", version);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            return Err(e.into());
        }
    }

    // Perform search
    match executor.search(query, limit) {
        Ok(results) => {
            if results.is_empty() {
                println!("   No results found for '{}'", query);
                return Ok(());
            }

            println!("   Found {} result(s):\n", results.len());

            // Output results based on format
            match format {
                "json" => {
                    let json = serde_json::to_string_pretty(&results)?;
                    println!("{}", json);
                }
                _ => {
                    // Text format (default)
                    for (i, result) in results.iter().enumerate() {
                        println!("{}. {}", i + 1, result.pname);
                        println!("   Version: {}", result.version);
                        if !result.description.is_empty() {
                            println!("   Description: {}", result.description);
                        }
                        println!();
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Search failed: {}", e);
            Err(e.into())
        }
    }
}

pub fn install(package: &str, yes: bool) -> Result<()> {
    println!("📦 Installing '{}'...", package);

    // Create bridged executor that uses WSL2
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    if let Err(e) = executor.check_nix_available() {
        eprintln!("❌ Error: {}", e);
        return Err(e.into());
    }

    // Confirm unless --yes flag
    if !yes {
        print!("   Proceed with installation? [y/N]: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("   Installation cancelled");
            return Ok(());
        }
    }

    // Perform installation
    match executor.install(package) {
        Ok(()) => {
            println!("✅ Successfully installed '{}'", package);
            Ok(())
        }
        Err(NixError::AlreadyInstalled(_)) => {
            println!("ℹ️  Package '{}' is already installed", package);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Installation failed: {}", e);
            Err(e.into())
        }
    }
}

pub fn remove(package: &str, yes: bool) -> Result<()> {
    println!("🗑️  Removing '{}'...", package);

    // Create bridged executor that uses WSL2
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    if let Err(e) = executor.check_nix_available() {
        eprintln!("❌ Error: {}", e);
        return Err(e.into());
    }

    // Confirm unless --yes flag
    if !yes {
        print!("   Proceed with removal? [y/N]: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("   Removal cancelled");
            return Ok(());
        }
    }

    // Perform removal
    match executor.remove(package) {
        Ok(()) => {
            println!("✅ Successfully removed '{}'", package);
            Ok(())
        }
        Err(NixError::NotInstalled(_)) => {
            println!("ℹ️  Package '{}' is not installed", package);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Removal failed: {}", e);
            Err(e.into())
        }
    }
}

pub fn list(detailed: bool, format: &str) -> Result<()> {
    println!("📋 Listing installed packages...");

    // Create bridged executor that uses WSL2
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    if let Err(e) = executor.check_nix_available() {
        eprintln!("❌ Error: {}", e);
        return Err(e.into());
    }

    // Get list of installed packages
    match executor.list() {
        Ok(packages) => {
            if packages.is_empty() {
                println!("   No packages installed");
                return Ok(());
            }

            println!("   {} package(s) installed:\n", packages.len());

            // Output based on format
            match format {
                "json" => {
                    let json = serde_json::to_string_pretty(&packages)?;
                    println!("{}", json);
                }
                _ => {
                    // Text format (default)
                    for (i, pkg) in packages.iter().enumerate() {
                        println!("{}. {}", i + 1, pkg.name);
                        if detailed {
                            println!("   Version: {}", pkg.version);
                            println!("   Store path: {}", pkg.store_path);
                        }
                        println!();
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to list packages: {}", e);
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
