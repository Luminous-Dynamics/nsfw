use anyhow::{Result, Context as AnyhowContext};
use std::path::PathBuf;
use std::fs;
use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::nix_ops::{BridgedNixExecutor, NixError, ErrorContext, types::SearchResult};
use crate::templates::{WrapperGenerator, PackageInfo, WrapperType};
use crate::wsl2::{RealWSL2Bridge, bridge::WSL2Bridge};
use crate::cache::SearchCache;
use crate::ui::{ProgressIndicator, OutputFormatter, MessageType};
use crate::package_cache::{PackageCache, CacheBuilder, CachedPackage};
use crate::config::Config;

/// Helper function to format NixError with context
fn format_nix_error(error: &NixError) -> String {
    let mut output = String::new();

    // Error message
    output.push_str(&OutputFormatter::format_message(
        MessageType::Error,
        &error.user_message()
    ));
    output.push('\n');

    // Suggestion if available
    if let Some(suggestion) = error.suggestion() {
        output.push('\n');
        output.push_str(&format!("{}\n{}",
            "💡 Suggestion:".bright_cyan().bold(),
            suggestion.bright_white()
        ));
        output.push('\n');
    }

    // Help URL if available
    if let Some(url) = error.help_url() {
        output.push('\n');
        output.push_str(&format!("{} {}",
            "📖 More info:".bright_cyan(),
            url.bright_blue().underline()
        ));
        output.push('\n');
    }

    output
}

/// Helper to spawn background cache update if needed
fn spawn_cache_update_if_needed(cache: PackageCache) {
    std::thread::spawn(move || {
        let bridge = RealWSL2Bridge::new();
        let builder = CacheBuilder::new(cache, bridge);

        if let Ok(true) = builder.needs_update() {
            log::info!("Starting background cache update");
            if let Err(e) = builder.build_from_nix_env() {
                log::warn!("Background cache update failed: {}", e);
            }
        }
    });
}

pub fn search(query: &str, limit: usize, format: &str, fuzzy: bool) -> Result<()> {
    use crate::performance::PerformanceTimer;

    // Start timing
    let timer = PerformanceTimer::start("search");

    // Show search header
    eprintln!("{}", OutputFormatter::format_section(&format!("Searching for '{}'", query)));

    // Try package cache first (local database - instant!)
    let pkg_cache = PackageCache::new()?;
    pkg_cache.initialize()?;

    if !pkg_cache.is_empty() {
        log::debug!("Checking package cache for '{}' (fuzzy: {})", query, fuzzy);
        let cached_packages = if fuzzy {
            pkg_cache.fuzzy_search(query, limit)?
        } else {
            pkg_cache.search(query, limit)?
        };

        if !cached_packages.is_empty() {
            log::info!("Found {} results in package cache", cached_packages.len());

            // Convert to SearchResult format
            let results: Vec<SearchResult> = cached_packages.iter().map(|p| SearchResult {
                pname: p.name.clone(),
                version: p.version.clone(),
                description: p.description.clone(),
            }).collect();

            // Output results
            if format == "json" {
                let json = serde_json::to_string_pretty(&results)?;
                println!("{}", json);
            } else {
                eprintln!("{}", OutputFormatter::format_message(MessageType::Success, &format!("⚡ Found {} result(s) (instant search!)", results.len())));
                print!("{}", OutputFormatter::format_search_results(&results, true));
            }

            // Start background cache update if needed
            spawn_cache_update_if_needed(pkg_cache);

            timer.finish();
            return Ok(());
        } else {
            log::debug!("No results in package cache, falling back to Nix search");
        }
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

            // Cache the results (search cache for this specific query)
            SearchCache::put(query, limit, results.clone());

            // Add results to package cache for future instant searches
            let cached_packages: Vec<CachedPackage> = results.iter().map(|r| CachedPackage {
                name: r.pname.clone(),
                version: r.version.clone(),
                description: r.description.clone(),
                attr_path: format!("nixpkgs.{}", r.pname),
                last_updated: chrono::Utc::now().timestamp(),
                search_count: 0,
            }).collect();

            if !cached_packages.is_empty() {
                if let Err(e) = pkg_cache.upsert_packages(&cached_packages) {
                    log::warn!("Failed to cache search results: {}", e);
                }
            }

            // Start background cache build if cache is empty (first-time user)
            if pkg_cache.is_empty() {
                log::info!("Starting background cache build for future instant searches");
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Info,
                    "💡 Building local package cache in background for instant future searches..."
                ));
                spawn_cache_update_if_needed(pkg_cache);
            }

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

pub fn install(package: &str, yes: bool, dry_run: bool) -> Result<()> {
    if dry_run {
        eprintln!("{}", OutputFormatter::format_section(&format!("DRY RUN: Would install '{}'", package)));
        eprintln!();
        eprintln!("{}", "The following actions would be performed:".bright_cyan());
        eprintln!("  {} Connect to WSL2", "→".bright_blue());
        eprintln!("  {} Check Nix availability", "→".bright_blue());
        eprintln!("  {} Search for package: {}", "→".bright_blue(), package.bright_yellow());
        eprintln!("  {} Install package via nix-env", "→".bright_blue());
        eprintln!("  {} Generate Windows wrapper scripts", "→".bright_blue());
        eprintln!();
        eprintln!("{}", "💡 No changes will be made (dry-run mode)".bright_cyan());
        eprintln!("   Remove --dry-run to perform actual installation");
        return Ok(());
    }

    eprintln!("{}", OutputFormatter::format_section(&format!("Installing '{}'", package)));

    // Create bridged executor that uses WSL2
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprint!("{}", format_nix_error(&e));
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

pub fn remove(package: &str, yes: bool, dry_run: bool) -> Result<()> {
    if dry_run {
        eprintln!("{}", OutputFormatter::format_section(&format!("DRY RUN: Would remove '{}'", package)));
        eprintln!();
        eprintln!("{}", "The following actions would be performed:".bright_cyan());
        eprintln!("  {} Connect to WSL2", "→".bright_blue());
        eprintln!("  {} Check Nix availability", "→".bright_blue());
        eprintln!("  {} Verify package is installed", "→".bright_blue());
        eprintln!("  {} Remove package via nix-env", "→".bright_blue());
        eprintln!("  {} Clean up Windows wrapper scripts", "→".bright_blue());
        eprintln!();
        eprintln!("{}", "💡 No changes will be made (dry-run mode)".bright_cyan());
        eprintln!("   Remove --dry-run to perform actual removal");
        return Ok(());
    }

    eprintln!("{}", OutputFormatter::format_section(&format!("Removing '{}'", package)));

    // Create bridged executor that uses WSL2
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprint!("{}", format_nix_error(&e));
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

/// Install multiple packages at once
pub fn install_batch(packages: &[String], yes: bool, dry_run: bool) -> Result<()> {
    use crate::package_cache::{PackageCache, HistoryAction};

    if dry_run {
        eprintln!("{}", OutputFormatter::format_section(&format!("DRY RUN: Would install {} package(s)", packages.len())));
        eprintln!();
        eprintln!("{}", "The following packages would be installed:".bright_cyan());
        for (i, pkg) in packages.iter().enumerate() {
            eprintln!("  {}. {}", (i + 1).to_string().bright_blue(), pkg.bright_yellow());
        }
        eprintln!();
        eprintln!("{}", "Actions that would be performed:".bright_cyan());
        eprintln!("  {} Connect to WSL2", "→".bright_blue());
        eprintln!("  {} Check Nix availability", "→".bright_blue());
        eprintln!("  {} For each package:", "→".bright_blue());
        eprintln!("    {} Search for package", "→".bright_blue());
        eprintln!("    {} Install package via nix-env", "→".bright_blue());
        eprintln!("    {} Generate Windows wrapper scripts", "→".bright_blue());
        eprintln!();
        eprintln!("{}", "💡 No changes will be made (dry-run mode)".bright_cyan());
        eprintln!("   Remove --dry-run to perform actual installation");
        return Ok(());
    }

    eprintln!("{}", OutputFormatter::format_section(&format!("Installing {} package(s)", packages.len())));
    eprintln!();
    for (i, pkg) in packages.iter().enumerate() {
        eprintln!("  {}. {}", (i + 1).to_string().bright_blue(), pkg.bright_yellow());
    }
    eprintln!();

    // Create bridged executor that uses WSL2
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Initialize cache for history tracking
    let cache = PackageCache::new().ok();
    if let Some(ref cache) = cache {
        let _ = cache.initialize();
    }

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprint!("{}", format_nix_error(&e));
        return Err(e.into());
    }
    progress.finish_and_clear();

    // Confirm unless --yes flag
    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Proceed with installation of {} package(s)?", packages.len()))
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("{}", OutputFormatter::format_message(MessageType::Info, "Installation cancelled"));
            return Ok(());
        }
    }

    // Install each package
    let mut success_count = 0;
    let mut already_installed_count = 0;
    let mut failed_packages: Vec<(String, String)> = Vec::new();
    let total_packages = packages.len();

    for (index, package) in packages.iter().enumerate() {
        let current_num = index + 1;
        let progress_pct = (current_num as f64 / total_packages as f64 * 100.0) as u32;

        eprintln!();
        eprintln!("{}", "─".repeat(60).bright_black());
        eprintln!("{} [{}/{}] {}% - {}",
            "📦".bright_cyan(),
            current_num.to_string().bright_white().bold(),
            total_packages.to_string().bright_white(),
            progress_pct.to_string().bright_cyan(),
            package.bright_yellow().bold()
        );
        eprintln!("{}", "─".repeat(60).bright_black());

        let progress = ProgressIndicator::spinner(&format!("Installing '{}'...", package));
        match executor.install(package) {
            Ok(()) => {
                progress.finish_and_clear();
                eprintln!("{}", OutputFormatter::format_message(MessageType::Success, &format!("✓ Successfully installed '{}'", package)));
                success_count += 1;

                // Record successful installation in history
                if let Some(ref cache) = cache {
                    let _ = cache.record_history(HistoryAction::Install, package, None, true, None);
                }
            }
            Err(NixError::AlreadyInstalled(_)) => {
                progress.finish_and_clear();
                eprintln!("{}", OutputFormatter::format_message(MessageType::Info, &format!("→ Package '{}' is already installed", package)));
                already_installed_count += 1;

                // Don't record already-installed as history (it's not a real operation)
            }
            Err(e) => {
                progress.finish_and_clear();
                eprintln!("{}", OutputFormatter::format_error_with_suggestion(
                    &format!("✗ Installation of '{}' failed: {}", package, e),
                    "Continuing with remaining packages..."
                ));

                // Record failed installation in history
                if let Some(ref cache) = cache {
                    let _ = cache.record_history(HistoryAction::Install, package, None, false, Some(&e.to_string()));
                }

                failed_packages.push((package.clone(), e.to_string()));
            }
        }
    }

    // Display summary
    eprintln!();
    eprintln!("{}", "═".repeat(60).bright_black());
    eprintln!();
    eprintln!("{}", "Installation Summary".bright_cyan().bold());
    eprintln!("  {} Succeeded: {}", "✓".bright_green(), success_count);
    if already_installed_count > 0 {
        eprintln!("  {} Already installed: {}", "→".bright_blue(), already_installed_count);
    }
    if !failed_packages.is_empty() {
        eprintln!("  {} Failed: {}", "✗".bright_red(), failed_packages.len());
        eprintln!();
        eprintln!("{}", "Failed packages:".bright_red());
        for (pkg, err) in &failed_packages {
            eprintln!("  • {}: {}", pkg.bright_yellow(), err.dimmed());
        }
    }

    if !failed_packages.is_empty() {
        Err(anyhow::anyhow!("{} package(s) failed to install", failed_packages.len()))
    } else {
        Ok(())
    }
}

/// Remove multiple packages at once
pub fn remove_batch(packages: &[String], yes: bool, dry_run: bool) -> Result<()> {
    use crate::package_cache::{PackageCache, HistoryAction};

    if dry_run {
        eprintln!("{}", OutputFormatter::format_section(&format!("DRY RUN: Would remove {} package(s)", packages.len())));
        eprintln!();
        eprintln!("{}", "The following packages would be removed:".bright_cyan());
        for (i, pkg) in packages.iter().enumerate() {
            eprintln!("  {}. {}", (i + 1).to_string().bright_blue(), pkg.bright_yellow());
        }
        eprintln!();
        eprintln!("{}", "Actions that would be performed:".bright_cyan());
        eprintln!("  {} Connect to WSL2", "→".bright_blue());
        eprintln!("  {} Check Nix availability", "→".bright_blue());
        eprintln!("  {} For each package:", "→".bright_blue());
        eprintln!("    {} Verify package is installed", "→".bright_blue());
        eprintln!("    {} Remove package via nix-env", "→".bright_blue());
        eprintln!("    {} Clean up Windows wrapper scripts", "→".bright_blue());
        eprintln!();
        eprintln!("{}", "💡 No changes will be made (dry-run mode)".bright_cyan());
        eprintln!("   Remove --dry-run to perform actual removal");
        return Ok(());
    }

    eprintln!("{}", OutputFormatter::format_section(&format!("Removing {} package(s)", packages.len())));
    eprintln!();
    for (i, pkg) in packages.iter().enumerate() {
        eprintln!("  {}. {}", (i + 1).to_string().bright_blue(), pkg.bright_yellow());
    }
    eprintln!();

    // Create bridged executor that uses WSL2
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Initialize cache for history tracking
    let cache = PackageCache::new().ok();
    if let Some(ref cache) = cache {
        let _ = cache.initialize();
    }

    // Check if Nix is available
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprint!("{}", format_nix_error(&e));
        return Err(e.into());
    }
    progress.finish_and_clear();

    // Confirm unless --yes flag
    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Proceed with removal of {} package(s)?", packages.len()))
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("{}", OutputFormatter::format_message(MessageType::Info, "Removal cancelled"));
            return Ok(());
        }
    }

    // Remove each package
    let mut success_count = 0;
    let mut not_installed_count = 0;
    let mut failed_packages: Vec<(String, String)> = Vec::new();
    let total_packages = packages.len();

    for (index, package) in packages.iter().enumerate() {
        let current_num = index + 1;
        let progress_pct = (current_num as f64 / total_packages as f64 * 100.0) as u32;

        eprintln!();
        eprintln!("{}", "─".repeat(60).bright_black());
        eprintln!("{} [{}/{}] {}% - {}",
            "🗑️".bright_cyan(),
            current_num.to_string().bright_white().bold(),
            total_packages.to_string().bright_white(),
            progress_pct.to_string().bright_cyan(),
            package.bright_yellow().bold()
        );
        eprintln!("{}", "─".repeat(60).bright_black());

        let progress = ProgressIndicator::spinner(&format!("Removing '{}'...", package));
        match executor.remove(package) {
            Ok(()) => {
                progress.finish_and_clear();
                eprintln!("{}", OutputFormatter::format_message(MessageType::Success, &format!("✓ Successfully removed '{}'", package)));
                success_count += 1;

                // Record successful removal in history
                if let Some(ref cache) = cache {
                    let _ = cache.record_history(HistoryAction::Remove, package, None, true, None);
                }
            }
            Err(NixError::NotInstalled(_)) => {
                progress.finish_and_clear();
                eprintln!("{}", OutputFormatter::format_message(MessageType::Warning, &format!("→ Package '{}' is not installed", package)));
                not_installed_count += 1;

                // Don't record not-installed as history (it's not a real operation)
            }
            Err(e) => {
                progress.finish_and_clear();
                eprintln!("{}", OutputFormatter::format_error_with_suggestion(
                    &format!("✗ Removal of '{}' failed: {}", package, e),
                    "Continuing with remaining packages..."
                ));

                // Record failed removal in history
                if let Some(ref cache) = cache {
                    let _ = cache.record_history(HistoryAction::Remove, package, None, false, Some(&e.to_string()));
                }

                failed_packages.push((package.clone(), e.to_string()));
            }
        }
    }

    // Display summary
    eprintln!();
    eprintln!("{}", "═".repeat(60).bright_black());
    eprintln!();
    eprintln!("{}", "Removal Summary".bright_cyan().bold());
    eprintln!("  {} Succeeded: {}", "✓".bright_green(), success_count);
    if not_installed_count > 0 {
        eprintln!("  {} Not installed: {}", "→".bright_yellow(), not_installed_count);
    }
    if !failed_packages.is_empty() {
        eprintln!("  {} Failed: {}", "✗".bright_red(), failed_packages.len());
        eprintln!();
        eprintln!("{}", "Failed packages:".bright_red());
        for (pkg, err) in &failed_packages {
            eprintln!("  • {}: {}", pkg.bright_yellow(), err.dimmed());
        }
    }

    if !failed_packages.is_empty() {
        Err(anyhow::anyhow!("{} package(s) failed to remove", failed_packages.len()))
    } else {
        Ok(())
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
        eprint!("{}", format_nix_error(&e));
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
    eprintln!("{}", OutputFormatter::format_section(&format!("Package Information: '{}'", package)));

    // Create bridged executor
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check Nix availability
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprint!("{}", format_nix_error(&e));
        return Err(e.into());
    }

    // Get package info
    progress.set_message(&format!("Fetching info for '{}'...", package));
    match executor.info(package) {
        Ok(pkg_info) => {
            progress.finish_and_clear();
            print!("{}", OutputFormatter::format_package_info(&pkg_info));
            Ok(())
        }
        Err(e @ NixError::PackageNotFound(_)) => {
            progress.finish_and_clear();
            eprint!("{}", format_nix_error(&e));
            Err(e.into())
        }
        Err(e) => {
            progress.finish_and_clear();
            eprint!("{}", format_nix_error(&e));
            Err(e.into())
        }
    }
}

pub fn update() -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section("Updating Package Database"));

    // Create bridged executor
    let progress = ProgressIndicator::spinner("Connecting to WSL2...");
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    // Check Nix availability
    progress.set_message("Checking Nix availability...");
    if let Err(e) = executor.check_nix_available() {
        progress.finish_and_clear();
        eprint!("{}", format_nix_error(&e));
        return Err(e.into());
    }

    // Update channels
    progress.set_message("Updating Nix channels... (this may take a few minutes)");
    match executor.update_channels() {
        Ok(()) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(MessageType::Success, "Channels updated successfully!"));

            // Clear the package cache so it rebuilds with new packages
            let pkg_cache = PackageCache::new()?;
            pkg_cache.initialize()?;

            if !pkg_cache.is_empty() {
                eprintln!("{}", OutputFormatter::format_message(MessageType::Info, "Package cache will be rebuilt on next search"));
                // Optional: trigger cache rebuild in background
                std::thread::spawn(move || {
                    let bridge = RealWSL2Bridge::new();
                    let builder = CacheBuilder::new(pkg_cache, bridge);
                    if let Err(e) = builder.build_from_nix_env() {
                        log::warn!("Background cache rebuild failed: {}", e);
                    }
                });
            }

            Ok(())
        }
        Err(e) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_error_with_suggestion(
                &format!("Channel update failed: {}", e),
                "Ensure you have internet connectivity and Nix is properly configured"
            ));
            Err(e.into())
        }
    }
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

pub fn setup(auto_yes: bool, interactive: bool) -> Result<()> {
    use crate::setup::SetupWizard;

    let wizard = SetupWizard::new(auto_yes, interactive);
    wizard.run()
}

pub fn install_completion(shell: &str) -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section("Installing Shell Completions"));

    match shell.to_lowercase().as_str() {
        "powershell" | "pwsh" => install_powershell_completion(),
        "bash" => install_bash_completion(),
        "zsh" => install_zsh_completion(),
        "fish" => install_fish_completion(),
        _ => {
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Error,
                &format!("Unknown shell: '{}'", shell)
            ));
            eprintln!("Supported shells: powershell, bash, zsh, fish");
            Ok(())
        }
    }
}

fn install_powershell_completion() -> Result<()> {
    use std::fs;
    use std::env;

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "📦 Installing PowerShell completions..."
    ));
    eprintln!();

    // Get the PowerShell profile directory
    let profile_dir = if let Ok(home) = env::var("USERPROFILE") {
        PathBuf::from(home).join("Documents").join("PowerShell")
    } else {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Error,
            "Could not determine PowerShell profile directory"
        ));
        return Ok(());
    };

    // Create completions directory
    let completions_dir = profile_dir.join("Completions");
    if !completions_dir.exists() {
        fs::create_dir_all(&completions_dir)?;
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Success,
            &format!("✓ Created directory: {}", completions_dir.display())
        ));
    }

    // Write the PowerShell completion script
    let completion_script = include_str!("../../completions/nsfw.ps1");
    let target_path = completions_dir.join("nsfw.ps1");

    fs::write(&target_path, completion_script)?;
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Success,
        &format!("✓ Installed completion script to: {}", target_path.display())
    ));

    // Check PowerShell profile
    let profile_path = profile_dir.join("Microsoft.PowerShell_profile.ps1");
    let import_line = format!("Import-Module \"{}\"", target_path.display());

    let mut needs_profile_update = true;
    if profile_path.exists() {
        let profile_content = fs::read_to_string(&profile_path)?;
        if profile_content.contains("Import-Module") && profile_content.contains("nsfw.ps1") {
            needs_profile_update = false;
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Info,
                "✓ Already configured in PowerShell profile"
            ));
        }
    }

    if needs_profile_update {
        // Create or append to profile
        let profile_addition = format!("\n# NSFW Tab Completions\n{}\n", import_line);

        if profile_path.exists() {
            let mut profile_content = fs::read_to_string(&profile_path)?;
            profile_content.push_str(&profile_addition);
            fs::write(&profile_path, profile_content)?;
        } else {
            fs::write(&profile_path, &profile_addition)?;
        }

        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Success,
            "✓ Added import to PowerShell profile"
        ));
    }

    eprintln!();
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Success,
        "🎉 PowerShell completions installed successfully!"
    ));
    eprintln!();
    eprintln!("To activate completions, restart PowerShell or run:");
    eprintln!("  {}", ". $PROFILE".bright_cyan());
    eprintln!();
    eprintln!("Now you can use Tab to autocomplete:");
    eprintln!("  • Commands: nsfw [Tab]");
    eprintln!("  • Packages: nsfw install [Tab]");
    eprintln!("  • Options: nsfw search --[Tab]");

    Ok(())
}

fn install_bash_completion() -> Result<()> {
    use std::fs;

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "📦 Installing Bash completions..."
    ));
    eprintln!();

    let completion_script = include_str!("../../completions/nsfw.bash");

    // Try system-wide locations first, then user location
    let mut target_paths = vec![
        PathBuf::from("/etc/bash_completion.d/nsfw"),
        PathBuf::from("/usr/local/share/bash-completion/completions/nsfw"),
    ];

    if let Some(home) = dirs::home_dir() {
        target_paths.push(home.join(".local/share/bash-completion/completions/nsfw"));
    }

    let mut installed = false;
    for target_path in &target_paths {
        if let Some(parent) = target_path.parent() {
            if (parent.exists() || fs::create_dir_all(parent).is_ok())
                && fs::write(target_path, completion_script).is_ok()
            {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Success,
                    &format!("✓ Installed completion script to: {}", target_path.display())
                ));
                installed = true;
                break;
            }
        }
    }

    if !installed {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Warning,
            "Could not install to system locations. Manual installation required."
        ));
        eprintln!();
        eprintln!("{}", "To install manually:".bright_cyan());
        eprintln!("1. Save the completion script from completions/nsfw.bash");
        eprintln!("2. Copy to one of these locations:");
        eprintln!("   - /etc/bash_completion.d/nsfw");
        eprintln!("   - /usr/local/share/bash-completion/completions/nsfw");
        eprintln!("   - ~/.local/share/bash-completion/completions/nsfw");
        eprintln!("3. Source it: source /etc/bash_completion");
    } else {
        eprintln!();
        eprintln!("{}", "To activate:".bright_cyan());
        eprintln!("  source /etc/bash_completion");
        eprintln!("  or restart your terminal");
    }

    eprintln!();
    Ok(())
}

fn install_zsh_completion() -> Result<()> {
    use std::fs;

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "📦 Installing Zsh completions..."
    ));
    eprintln!();

    let completion_script = include_str!("../../completions/nsfw.zsh");

    // Try to find a location in $fpath
    let mut target_paths = vec![
        PathBuf::from("/usr/local/share/zsh/site-functions/_nsfw"),
    ];

    if let Some(home) = dirs::home_dir() {
        target_paths.push(home.join(".local/share/zsh/site-functions/_nsfw"));
        target_paths.push(home.join(".zsh/completions/_nsfw"));
    }

    let mut installed = false;
    for target_path in &target_paths {
        if let Some(parent) = target_path.parent() {
            if (parent.exists() || fs::create_dir_all(parent).is_ok())
                && fs::write(target_path, completion_script).is_ok()
            {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Success,
                    &format!("✓ Installed completion script to: {}", target_path.display())
                ));
                installed = true;
                break;
            }
        }
    }

    if !installed {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Warning,
            "Could not install to system locations. Manual installation required."
        ));
        eprintln!();
        eprintln!("{}", "To install manually:".bright_cyan());
        eprintln!("1. Create a directory: mkdir -p ~/.zsh/completions");
        eprintln!("2. Save completions/nsfw.zsh to ~/.zsh/completions/_nsfw");
        eprintln!("3. Add to ~/.zshrc:");
        eprintln!("   fpath=(~/.zsh/completions $fpath)");
        eprintln!("   autoload -Uz compinit && compinit");
    } else {
        eprintln!();
        eprintln!("{}", "To activate:".bright_cyan());
        eprintln!("  Restart your terminal or run:");
        eprintln!("  autoload -Uz compinit && compinit");
    }

    eprintln!();
    Ok(())
}

fn install_fish_completion() -> Result<()> {
    use std::fs;

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "📦 Installing Fish completions..."
    ));
    eprintln!();

    let completion_script = include_str!("../../completions/nsfw.fish");

    // Fish completions location
    let target_path = dirs::home_dir()
        .map(|h| h.join(".config/fish/completions/nsfw.fish"));

    if let Some(target_path) = target_path {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
            fs::write(&target_path, completion_script)?;

            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Success,
                &format!("✓ Installed completion script to: {}", target_path.display())
            ));
            eprintln!();
            eprintln!("{}", "Completions are active immediately!".bright_green());
            eprintln!("  (Fish automatically loads completions from ~/.config/fish/completions/)");
        }
    } else {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Error,
            "Could not determine home directory"
        ));
    }

    eprintln!();
    Ok(())
}

pub fn cache_stats() -> Result<()> {
    use crate::package_cache::CacheHealth;

    eprintln!("{}", OutputFormatter::format_section("Package Cache Statistics"));

    let cache = PackageCache::new()?;
    cache.initialize()?;

    // Get cache statistics
    let stats = cache.stats()?;
    let size = cache.get_size().unwrap_or(0);
    let age = cache.get_age_seconds()?;
    let health = cache.get_health()?;
    let effectiveness = cache.get_effectiveness()?;

    // Format health status with color
    let (health_text, health_color) = match health {
        CacheHealth::Empty => ("Empty", "bright_red"),
        CacheHealth::Fresh => ("Fresh", "bright_green"),
        CacheHealth::Good => ("Good", "green"),
        CacheHealth::Stale => ("Stale", "yellow"),
        CacheHealth::Outdated => ("Outdated", "red"),
    };

    let health_display = match health_color {
        "bright_green" => health_text.bright_green().to_string(),
        "green" => health_text.green().to_string(),
        "yellow" => health_text.yellow().to_string(),
        "red" => health_text.red().to_string(),
        "bright_red" => health_text.bright_red().to_string(),
        _ => health_text.to_string(),
    };

    // Format size nicely
    let size_mb = size as f64 / 1_048_576.0;
    let size_str = if size_mb < 1.0 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else {
        format!("{:.2} MB", size_mb)
    };

    // Format age nicely
    let age_str = if let Some(age_secs) = age {
        if age_secs < 60 {
            format!("{} seconds ago", age_secs)
        } else if age_secs < 3600 {
            format!("{} minutes ago", age_secs / 60)
        } else if age_secs < 86400 {
            format!("{} hours ago", age_secs / 3600)
        } else {
            format!("{} days ago", age_secs / 86400)
        }
    } else {
        "Never".to_string()
    };

    // === OVERVIEW SECTION ===
    eprintln!("\n{}", "  Overview".bright_white().bold());
    eprintln!("  {}", "─".repeat(58).bright_black());
    eprintln!("  {:<20} {}", "Status:".bright_white(), health_display);
    eprintln!("  {:<20} {}", "Total Packages:".bright_white(), stats.total_packages.to_string().bright_cyan());
    eprintln!("  {:<20} {}", "Database Size:".bright_white(), size_str.bright_cyan());
    eprintln!("  {:<20} {}", "Last Updated:".bright_white(), age_str.bright_cyan());

    // === USAGE STATISTICS ===
    if stats.total_packages > 0 {
        eprintln!("\n{}", "  Usage Statistics".bright_white().bold());
        eprintln!("  {}", "─".repeat(58).bright_black());
        eprintln!("  {:<20} {}", "Total Searches:".bright_white(), stats.total_searches.to_string().bright_cyan());
        eprintln!("  {:<20} {}", "Packages Searched:".bright_white(),
            format!("{} / {}", stats.packages_with_searches, stats.total_packages).bright_cyan());
        eprintln!("  {:<20} {}", "Cache Effectiveness:".bright_white(),
            format!("{:.1}%", effectiveness).bright_cyan());

        if stats.total_searches > 0 {
            let avg_searches = stats.total_searches as f64 / stats.packages_with_searches.max(1) as f64;
            eprintln!("  {:<20} {}", "Avg Searches/Package:".bright_white(),
                format!("{:.1}", avg_searches).bright_cyan());
        }
    }

    // === CACHE DETAILS ===
    if stats.total_packages > 0 {
        eprintln!("\n{}", "  Cache Details".bright_white().bold());
        eprintln!("  {}", "─".repeat(58).bright_black());
        eprintln!("  {:<20} {}", "Avg Description Len:".bright_white(),
            format!("{:.0} chars", stats.average_description_length).bright_cyan());

        // Show age range if packages have different timestamps
        if let (Some(oldest), Some(newest)) = (stats.oldest_package, stats.newest_package) {
            if oldest != newest {
                let age_range_days = (newest - oldest) / 86400;
                eprintln!("  {:<20} {}", "Cache Age Range:".bright_white(),
                    format!("{} days", age_range_days).bright_cyan());
            }
        }

        eprintln!("  {:<20} {}", "Location:".bright_white(),
            cache.get_path().display().to_string().bright_black());
    }

    // === MOST SEARCHED PACKAGES ===
    if stats.total_packages > 0 {
        if let Ok(popular) = cache.get_popular(10) {
            if !popular.is_empty() {
                eprintln!("\n{}", "  Most Searched Packages".bright_white().bold());
                eprintln!("  {}", "─".repeat(58).bright_black());

                for (i, pkg) in popular.iter().enumerate() {
                    let rank = format!("{:>2}.", i + 1).bright_black();
                    let count_display = format!("({} searches)", pkg.search_count).bright_black();
                    eprintln!("  {} {:<35} {}",
                        rank,
                        pkg.name.bright_green(),
                        count_display
                    );
                }
            }
        }
    }

    eprintln!();

    // === RECOMMENDATIONS ===
    if stats.total_packages == 0 {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            "Cache is empty. Run 'nsfw search <query>' to build it automatically."
        ));
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            "Or run 'nsfw cache rebuild' to build the complete cache (~80,000 packages)."
        ));
    } else {
        match health {
            CacheHealth::Fresh => {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Success,
                    "✓ Cache is fresh and up-to-date!"
                ));
            }
            CacheHealth::Good => {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Info,
                    "Cache is in good condition. No action needed."
                ));
            }
            CacheHealth::Stale => {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Warning,
                    "Cache is getting old. Consider running 'nsfw update' to refresh packages."
                ));
            }
            CacheHealth::Outdated => {
                eprintln!("{}", OutputFormatter::format_message(
                    MessageType::Warning,
                    "⚠ Cache is outdated! Run 'nsfw cache rebuild' for latest packages."
                ));
            }
            CacheHealth::Empty => {}
        }

        // Additional recommendations based on effectiveness
        if stats.total_packages > 100 && effectiveness < 1.0 {
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Info,
                &format!("Tip: Only {:.1}% of cached packages have been searched. Cache is highly underutilized.", effectiveness)
            ));
        }
    }

    Ok(())
}

pub fn cache_clear() -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section("Clear Package Cache"));

    let cache = PackageCache::new()?;
    cache.initialize()?;

    // Get stats before clearing
    let stats_before = cache.stats()?;

    if stats_before.total_packages == 0 {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            "Cache is already empty."
        ));
        return Ok(());
    }

    // Confirm with user
    use dialoguer::Confirm;
    let confirmed = Confirm::new()
        .with_prompt(format!("Clear {} cached packages?", stats_before.total_packages))
        .default(false)
        .interact()?;

    if !confirmed {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            "Cache clear cancelled."
        ));
        return Ok(());
    }

    // Clear the cache
    let progress = ProgressIndicator::spinner("Clearing cache...");
    cache.clear()?;
    progress.finish_and_clear();

    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Success,
        &format!("✓ Cleared {} packages from cache", stats_before.total_packages)
    ));

    eprintln!("\nThe cache will be rebuilt on your next search.");

    Ok(())
}

pub fn cache_rebuild() -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section("Rebuild Package Cache"));

    let cache = PackageCache::new()?;
    cache.initialize()?;

    // Clear existing cache
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "Clearing existing cache..."
    ));
    cache.clear()?;

    // Rebuild cache
    eprintln!("{}", OutputFormatter::format_message(
        MessageType::Info,
        "Rebuilding cache from nixpkgs..."
    ));
    eprintln!();
    eprintln!("  This may take 2-10 minutes depending on your system.");
    eprintln!("  Future searches will be instant!");
    eprintln!();

    let progress = ProgressIndicator::spinner("Building cache from Nix packages...");

    let bridge = RealWSL2Bridge::new();
    let builder = CacheBuilder::new(cache.clone(), bridge);

    match builder.build_from_nix_env() {
        Ok(count) => {
            progress.finish_and_clear();

            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Success,
                &format!("✓ Cache rebuilt successfully with {} packages", count)
            ));

            Ok(())
        }
        Err(e) => {
            progress.finish_and_clear();
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Error,
                &format!("Cache rebuild failed: {}", e)
            ));
            Err(e)
        }
    }
}

pub fn history(limit: usize, package_filter: Option<&str>, show_stats: bool) -> Result<()> {
    use crate::package_cache::HistoryAction;

    let cache = PackageCache::new()?;
    cache.initialize()?;

    if show_stats {
        // Show statistics instead of history entries
        eprintln!("{}", OutputFormatter::format_section("Installation History Statistics"));

        let stats = cache.get_history_stats()?;

        if stats.total_operations == 0 {
            eprintln!();
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Info,
                "No installation history recorded yet."
            ));
            return Ok(());
        }

        // Calculate success rate
        let success_rate = if stats.total_operations > 0 {
            (stats.successful_operations as f64 / stats.total_operations as f64) * 100.0
        } else {
            0.0
        };

        // Format last operation time
        let last_op_str = if let Some(last_op) = stats.last_operation {
            let now = chrono::Utc::now().timestamp();
            let age_secs = now - last_op;

            if age_secs < 60 {
                format!("{} seconds ago", age_secs)
            } else if age_secs < 3600 {
                format!("{} minutes ago", age_secs / 60)
            } else if age_secs < 86400 {
                format!("{} hours ago", age_secs / 3600)
            } else {
                format!("{} days ago", age_secs / 86400)
            }
        } else {
            "Never".to_string()
        };

        eprintln!("\n{}", "  Summary".bright_white().bold());
        eprintln!("  {}", "─".repeat(58).bright_black());
        eprintln!("  {:<20} {}", "Total Operations:".bright_white(),
            stats.total_operations.to_string().bright_cyan());
        eprintln!("  {:<20} {} ({})",
            "Successful:".bright_white(),
            stats.successful_operations.to_string().bright_green(),
            format!("{:.1}%", success_rate).bright_green());
        eprintln!("  {:<20} {}", "Failed:".bright_white(),
            stats.failed_operations.to_string().bright_red());
        eprintln!("  {:<20} {}", "Last Operation:".bright_white(),
            last_op_str.bright_cyan());

        eprintln!("\n{}", "  Operations by Type".bright_white().bold());
        eprintln!("  {}", "─".repeat(58).bright_black());
        eprintln!("  {:<20} {}", "Installs:".bright_white(),
            stats.total_installs.to_string().bright_cyan());
        eprintln!("  {:<20} {}", "Removals:".bright_white(),
            stats.total_removes.to_string().bright_cyan());

        eprintln!();
        return Ok(());
    }

    // Show history entries
    let entries = if let Some(pkg) = package_filter {
        eprintln!("{}", OutputFormatter::format_section(&format!("Installation History for '{}'", pkg)));
        cache.get_package_history(pkg, limit)?
    } else {
        eprintln!("{}", OutputFormatter::format_section("Installation History"));
        cache.get_history(limit)?
    };

    if entries.is_empty() {
        eprintln!();
        if let Some(pkg) = package_filter {
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Info,
                &format!("No history found for package '{}'", pkg)
            ));
        } else {
            eprintln!("{}", OutputFormatter::format_message(
                MessageType::Info,
                "No installation history recorded yet."
            ));
        }
        return Ok(());
    }

    eprintln!("\n{} recent operation(s):", entries.len());
    eprintln!();

    for entry in &entries {
        // Format timestamp
        let dt = chrono::DateTime::from_timestamp(entry.timestamp, 0)
            .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH);
        let time_str = dt.format("%Y-%m-%d %H:%M:%S").to_string();

        // Format action with color and emoji
        let (action_display, action_emoji) = match entry.action {
            HistoryAction::Install => (entry.action.as_str().bright_green(), "📦"),
            HistoryAction::Remove => (entry.action.as_str().bright_red(), "🗑️"),
            HistoryAction::Upgrade => (entry.action.as_str().bright_yellow(), "⬆️"),
            HistoryAction::Update => (entry.action.as_str().bright_blue(), "🔄"),
        };

        // Format status
        let status_display = if entry.success {
            "✓".bright_green()
        } else {
            "✗".bright_red()
        };

        // Print entry
        eprintln!("  {} {} {} {} {}",
            time_str.bright_black(),
            action_emoji,
            action_display,
            entry.package_name.bright_white().bold(),
            status_display
        );

        // Show version if available
        if let Some(ref version) = entry.version {
            eprintln!("       {} {}", "version:".bright_black(), version.bright_cyan());
        }

        // Show error message if failed
        if !entry.success {
            if let Some(ref err_msg) = entry.error_message {
                eprintln!("       {} {}", "error:".bright_red(), err_msg.bright_black());
            }
        }

        eprintln!();
    }

    // Show hint for more results
    if entries.len() == limit && package_filter.is_none() {
        eprintln!("{}", OutputFormatter::format_message(
            MessageType::Info,
            &format!("Showing last {} operations. Use --limit to show more.", limit)
        ));
    }

    Ok(())
}

pub fn doctor() -> Result<()> {
    eprintln!("{}", OutputFormatter::format_section("System Health Check"));
    eprintln!();
    eprintln!("Running diagnostics...");
    eprintln!();

    let mut issues_found = 0;
    let mut checks_passed = 0;

    // Check 1: WSL2 availability
    eprintln!("{}", "1. Checking WSL2...".bright_white());
    let bridge = RealWSL2Bridge::new();
    if bridge.is_available() {
        eprintln!("   {} WSL2 is installed and available", "✓".bright_green());
        checks_passed += 1;

        // Check WSL version
        match std::process::Command::new("wsl").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                if version.contains("WSL version") || version.contains("2.") {
                    eprintln!("   {} WSL2 version detected", "✓".bright_green());
                }
            }
            _ => {}
        }
    } else {
        eprintln!("   {} WSL2 is not available", "✗".bright_red());
        eprintln!("     {}: Install WSL2 with: {}",
            "Fix".bright_cyan(),
            "wsl --install".bright_yellow()
        );
        issues_found += 1;
    }
    eprintln!();

    // Check 2: Nix availability
    eprintln!("{}", "2. Checking Nix installation...".bright_white());
    if bridge.is_available() {
        let executor = BridgedNixExecutor::new(bridge.clone());
        match executor.check_nix_available() {
            Ok(version) => {
                eprintln!("   {} Nix is installed: {}", "✓".bright_green(), version);
                checks_passed += 1;
            }
            Err(_) => {
                eprintln!("   {} Nix is not installed in WSL2", "✗".bright_red());
                eprintln!("     {}: Run {}",
                    "Fix".bright_cyan(),
                    "nsfw setup".bright_yellow()
                );
                issues_found += 1;
            }
        }
    } else {
        eprintln!("   {} Cannot check (WSL2 not available)", "⊘".bright_yellow());
    }
    eprintln!();

    // Check 3: Package cache
    eprintln!("{}", "3. Checking package cache...".bright_white());
    match PackageCache::new() {
        Ok(cache) => {
            if let Ok(()) = cache.initialize() {
                let stats = cache.stats()?;
                if stats.total_packages > 0 {
                    eprintln!("   {} Package cache is healthy ({} packages)",
                        "✓".bright_green(),
                        stats.total_packages
                    );
                    checks_passed += 1;

                    // Check cache age
                    if let Ok(Some(age_secs)) = cache.get_age_seconds() {
                        if age_secs > 2592000 { // 30 days
                            eprintln!("   {} Cache is over 30 days old", "⚠".bright_yellow());
                            eprintln!("     {}: Run {}",
                                "Suggestion".bright_cyan(),
                                "nsfw update".bright_yellow()
                            );
                        }
                    }
                } else {
                    eprintln!("   {} Package cache is empty", "⚠".bright_yellow());
                    eprintln!("     {}: Run any search to build cache: {}",
                        "Fix".bright_cyan(),
                        "nsfw search firefox".bright_yellow()
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("   {} Cache error: {}", "✗".bright_red(), e);
            issues_found += 1;
        }
    }
    eprintln!();

    // Check 4: Permissions
    eprintln!("{}", "4. Checking permissions...".bright_white());
    let cache_dir = dirs::cache_dir();
    if let Some(dir) = cache_dir {
        let nsfw_cache = dir.join("nsfw");
        if nsfw_cache.exists() {
            match std::fs::metadata(&nsfw_cache) {
                Ok(metadata) => {
                    if !metadata.permissions().readonly() {
                        eprintln!("   {} Cache directory is writable", "✓".bright_green());
                        checks_passed += 1;
                    } else {
                        eprintln!("   {} Cache directory is read-only", "✗".bright_red());
                        issues_found += 1;
                    }
                }
                Err(e) => {
                    eprintln!("   {} Cannot check permissions: {}", "⚠".bright_yellow(), e);
                }
            }
        } else {
            eprintln!("   {} Cache directory will be created on first use", "ℹ".bright_blue());
        }
    }
    eprintln!();

    // Check 5: Disk space
    eprintln!("{}", "5. Checking disk space...".bright_white());
    if let Ok(cache) = PackageCache::new() {
        if let Ok(size) = cache.get_size() {
            let size_mb = size as f64 / 1_048_576.0;
            eprintln!("   {} Cache using {:.2} MB", "ℹ".bright_blue(), size_mb);

            // Check if cache is unusually large
            if size_mb > 1000.0 {
                eprintln!("   {} Cache is very large (> 1 GB)", "⚠".bright_yellow());
                eprintln!("     {}: Consider running {}",
                    "Suggestion".bright_cyan(),
                    "nsfw cache clear".bright_yellow()
                );
            }
        }
    }
    eprintln!();

    // Summary
    eprintln!("{}", "═".repeat(60).bright_black());
    eprintln!();

    if issues_found == 0 {
        eprintln!("{} {} System is healthy! ({} checks passed)",
            "🎉".bright_green(),
            "Success:".bright_green().bold(),
            checks_passed
        );
        eprintln!();
        eprintln!("Your NSFW installation is working correctly.");
    } else {
        eprintln!("{} {} Found {} issue(s) ({} checks passed)",
            "⚠".bright_yellow(),
            "Warning:".bright_yellow().bold(),
            issues_found,
            checks_passed
        );
        eprintln!();
        eprintln!("Please address the issues above for optimal performance.");
        eprintln!();
        eprintln!("Quick fixes:");
        eprintln!("  • Install WSL2: {}", "wsl --install".bright_yellow());
        eprintln!("  • Setup Nix: {}", "nsfw setup".bright_yellow());
        eprintln!("  • Build cache: {}", "nsfw search firefox".bright_yellow());
    }

    eprintln!();
    eprintln!("For more help, visit: {}",
        "https://github.com/Luminous-Dynamics/nsfw".bright_blue().underline()
    );

    Ok(())
}

// ============================================================================
// Configuration Commands
// ============================================================================

/// Show all configuration settings
pub fn config_show() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Configuration Settings".bright_cyan().bold());
    println!();

    for key in Config::keys() {
        let value = config.get(key)?;
        let description = Config::key_description(key);

        println!("  {} = {}",
            key.bright_yellow(),
            value.bright_white()
        );
        println!("    {}", description.dimmed());
        println!();
    }

    let config_path = Config::config_path()?;
    println!("{} {}",
        "Config file:".bright_cyan(),
        config_path.display().to_string().bright_blue()
    );

    Ok(())
}

/// Get a specific configuration value
pub fn config_get(key: &str) -> Result<()> {
    let config = Config::load()?;
    let value = config.get(key)?;

    println!("{}", value);

    Ok(())
}

/// Set a configuration value
pub fn config_set(key: &str, value: &str) -> Result<()> {
    let mut config = Config::load()?;

    println!("{} {} {} {}",
        "Setting".bright_cyan(),
        key.bright_yellow(),
        "to".bright_cyan(),
        value.bright_green()
    );

    config.set(key, value)?;
    config.save()?;

    println!();
    println!("{} Configuration updated successfully!",
        "✓".bright_green().bold()
    );

    let config_path = Config::config_path()?;
    println!("  File: {}", config_path.display().to_string().dimmed());

    Ok(())
}

/// Reset configuration to defaults
pub fn config_reset() -> Result<()> {
    println!("{}", "Resetting configuration to defaults...".bright_yellow());

    let default_config = Config::default();
    default_config.save()?;

    println!();
    println!("{} Configuration reset successfully!",
        "✓".bright_green().bold()
    );

    let config_path = Config::config_path()?;
    println!("  File: {}", config_path.display().to_string().dimmed());

    Ok(())
}

/// Show configuration file path
pub fn config_path() -> Result<()> {
    let config_path = Config::config_path()?;

    println!("{}", config_path.display());

    Ok(())
}

/// List all available configuration keys
pub fn config_keys() -> Result<()> {
    println!("{}", "Available Configuration Keys".bright_cyan().bold());
    println!();

    for key in Config::keys() {
        let description = Config::key_description(key);

        println!("  {}", key.bright_yellow());
        println!("    {}", description.dimmed());
        println!();
    }

    println!("{}", "Usage:".bright_cyan().bold());
    println!("  {} {}", "nsfw config get".bright_white(), "<key>".dimmed());
    println!("  {} {} {}", "nsfw config set".bright_white(), "<key>".dimmed(), "<value>".dimmed());

    Ok(())
}

// ============================================================================
// Package Management Commands
// ============================================================================

/// Exported package list format
#[derive(Debug, Serialize, Deserialize)]
struct ExportedPackages {
    /// Timestamp when exported
    exported_at: String,
    /// NSFW version
    version: String,
    /// List of installed packages
    packages: Vec<ExportedPackage>,
}

/// Single exported package
#[derive(Debug, Serialize, Deserialize)]
struct ExportedPackage {
    /// Package name
    name: String,
    /// Package version (if available)
    version: Option<String>,
}

/// Upgrade installed package(s) to latest version
pub fn upgrade(package: Option<&str>, yes: bool, dry_run: bool) -> Result<()> {
    if dry_run {
        if let Some(pkg_name) = package {
            // Single package dry-run
            eprintln!("{}", OutputFormatter::format_section(&format!("DRY RUN: Would upgrade '{}'", pkg_name)));
            eprintln!();
            eprintln!("{}", "The following actions would be performed:".bright_cyan());
            eprintln!("  {} Connect to WSL2", "→".bright_blue());
            eprintln!("  {} Check package current version", "→".bright_blue());
            eprintln!("  {} Remove old version of: {}", "→".bright_blue(), pkg_name.bright_yellow());
            eprintln!("  {} Install latest version of: {}", "→".bright_blue(), pkg_name.bright_yellow());
            eprintln!("  {} Update Windows wrapper scripts", "→".bright_blue());
            eprintln!();
        } else {
            // All packages dry-run
            eprintln!("{}", OutputFormatter::format_section("DRY RUN: Would upgrade all packages"));
            eprintln!();
            eprintln!("{}", "The following actions would be performed:".bright_cyan());
            eprintln!("  {} Connect to WSL2", "→".bright_blue());
            eprintln!("  {} List all installed packages", "→".bright_blue());
            eprintln!("  {} For each package:", "→".bright_blue());
            eprintln!("    {} Remove old version", "→".bright_blue());
            eprintln!("    {} Install latest version", "→".bright_blue());
            eprintln!("  {} Update Windows wrapper scripts", "→".bright_blue());
            eprintln!();
        }
        eprintln!("{}", "💡 No changes will be made (dry-run mode)".bright_cyan());
        eprintln!("   Remove --dry-run to perform actual upgrade");
        return Ok(());
    }

    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    if let Some(pkg_name) = package {
        // Upgrade single package
        println!("{} {}",
            "Upgrading package:".bright_cyan(),
            pkg_name.bright_yellow()
        );

        if !yes {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!("Upgrade {} to latest version?", pkg_name))
                .default(true)
                .interact()?;

            if !confirmed {
                println!("Upgrade cancelled.");
                return Ok(());
            }
        }

        // For now, upgrade = remove + install latest
        // First remove the old version
        match executor.remove(pkg_name) {
            Ok(_) => {
                println!("{} Removed old version",
                    "✓".bright_green()
                );
            }
            Err(e) => {
                eprint!("{}", format_nix_error(&e));
                return Err(e.into());
            }
        }

        // Then install the latest version
        match executor.install(pkg_name) {
            Ok(_) => {
                println!("{} Installed latest version",
                    "✓".bright_green()
                );
            }
            Err(e) => {
                eprint!("{}", format_nix_error(&e));
                return Err(e.into());
            }
        }

        println!();
        println!("{} Successfully upgraded {}!",
            "✓".bright_green().bold(),
            pkg_name.bright_yellow()
        );

    } else {
        // Upgrade all packages
        println!("{}", "Upgrading all packages...".bright_cyan());

        // Get list of installed packages
        let installed = match executor.list() {
            Ok(packages) => packages,
            Err(e) => {
                eprint!("{}", format_nix_error(&e));
                return Err(e.into());
            }
        };

        if installed.is_empty() {
            println!("No packages installed.");
            return Ok(());
        }

        println!("Found {} installed packages", installed.len());

        if !yes {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!("Upgrade {} packages to latest versions?", installed.len()))
                .default(true)
                .interact()?;

            if !confirmed {
                println!("Upgrade cancelled.");
                return Ok(());
            }
        }

        let mut success_count = 0;
        let mut fail_count = 0;

        for pkg in &installed {
            print!("Upgrading {}... ", pkg.name.bright_yellow());

            // Remove old version
            match executor.remove(&pkg.name) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", "Failed to remove".bright_red());
                    eprintln!("  Error: {}", e);
                    fail_count += 1;
                    continue;
                }
            }

            // Install latest version
            match executor.install(&pkg.name) {
                Ok(_) => {
                    println!("{}", "✓".bright_green());
                    success_count += 1;
                }
                Err(e) => {
                    println!("{}", "Failed".bright_red());
                    eprintln!("  Error: {}", e);
                    fail_count += 1;
                }
            }
        }

        println!();
        println!("{}", "Upgrade Summary".bright_cyan().bold());
        println!("  {} Succeeded: {}", "✓".bright_green(), success_count);
        if fail_count > 0 {
            println!("  {} Failed: {}", "✗".bright_red(), fail_count);
        }
    }

    Ok(())
}

/// Export installed packages to a file
pub fn export(output: &str, format: &str) -> Result<()> {
    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    println!("{} {}",
        "Exporting packages to:".bright_cyan(),
        output.bright_yellow()
    );

    // Get list of installed packages
    let installed = match executor.list() {
        Ok(packages) => packages,
        Err(e) => {
            eprint!("{}", format_nix_error(&e));
            return Err(e.into());
        }
    };

    if installed.is_empty() {
        println!("No packages installed to export.");
        return Ok(());
    }

    // Create export structure
    let exported = ExportedPackages {
        exported_at: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        packages: installed.iter().map(|pkg| ExportedPackage {
            name: pkg.name.clone(),
            version: Some(pkg.version.clone()),
        }).collect(),
    };

    // Serialize based on format
    let content = match format {
        "json" => {
            serde_json::to_string_pretty(&exported)
                .context("Failed to serialize to JSON")?
        }
        "toml" => {
            toml::to_string_pretty(&exported)
                .context("Failed to serialize to TOML")?
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown format: {}. Use 'json' or 'toml'", format));
        }
    };

    // Write to file
    fs::write(output, content)
        .context("Failed to write export file")?;

    println!();
    println!("{} Exported {} packages",
        "✓".bright_green().bold(),
        installed.len()
    );
    println!("  Format: {}", format.dimmed());
    println!("  File: {}", output.dimmed());

    Ok(())
}

/// Import and install packages from a file
pub fn import(file: &str, yes: bool, dry_run: bool) -> Result<()> {
    println!("{} {}",
        "Importing packages from:".bright_cyan(),
        file.bright_yellow()
    );

    // Read file
    let content = fs::read_to_string(file)
        .context("Failed to read import file")?;

    // Try to parse as JSON first, then TOML
    let exported: ExportedPackages = if let Ok(parsed) = serde_json::from_str(&content) {
        parsed
    } else if let Ok(parsed) = toml::from_str(&content) {
        parsed
    } else {
        return Err(anyhow::anyhow!("Failed to parse file as JSON or TOML"));
    };

    println!("Found {} packages to install", exported.packages.len());
    println!("  Exported at: {}", exported.exported_at.dimmed());
    println!("  Exported from NSFW version: {}", exported.version.dimmed());
    println!();

    if dry_run {
        eprintln!("{}", OutputFormatter::format_section(&format!("DRY RUN: Would import {} packages", exported.packages.len())));
        eprintln!();
        eprintln!("{}", "The following packages would be installed:".bright_cyan());
        for (i, pkg) in exported.packages.iter().enumerate() {
            eprintln!("  {}. {} {}",
                (i + 1).to_string().bright_blue(),
                pkg.name.bright_yellow(),
                if let Some(ref ver) = pkg.version {
                    format!("({})", ver).dimmed().to_string()
                } else {
                    "".to_string()
                }
            );
        }
        eprintln!();
        eprintln!("{}", "Actions that would be performed:".bright_cyan());
        eprintln!("  {} Connect to WSL2", "→".bright_blue());
        eprintln!("  {} Check which packages are already installed", "→".bright_blue());
        eprintln!("  {} Install missing packages via nix-env", "→".bright_blue());
        eprintln!("  {} Generate Windows wrapper scripts", "→".bright_blue());
        eprintln!();
        eprintln!("{}", "💡 No changes will be made (dry-run mode)".bright_cyan());
        eprintln!("   Remove --dry-run to perform actual import");
        return Ok(());
    }

    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Install {} packages?", exported.packages.len()))
            .default(true)
            .interact()?;

        if !confirmed {
            println!("Import cancelled.");
            return Ok(());
        }
    }

    let bridge = RealWSL2Bridge::new();
    let executor = BridgedNixExecutor::new(bridge);

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut skipped_count = 0;

    for pkg in &exported.packages {
        print!("Installing {}... ", pkg.name.bright_yellow());

        // Check if already installed
        if let Ok(installed) = executor.list() {
            if installed.iter().any(|p| p.name == pkg.name) {
                println!("{}", "Already installed".bright_blue());
                skipped_count += 1;
                continue;
            }
        }

        match executor.install(&pkg.name) {
            Ok(_) => {
                println!("{}", "✓".bright_green());
                success_count += 1;
            }
            Err(e) => {
                println!("{}", "Failed".bright_red());
                eprintln!("  Error: {}", e);
                fail_count += 1;
            }
        }
    }

    println!();
    println!("{}", "Import Summary".bright_cyan().bold());
    println!("  {} Installed: {}", "✓".bright_green(), success_count);
    if skipped_count > 0 {
        println!("  {} Skipped (already installed): {}", "→".bright_blue(), skipped_count);
    }
    if fail_count > 0 {
        println!("  {} Failed: {}", "✗".bright_red(), fail_count);
    }

    Ok(())
}
