use clap::{Parser, Subcommand};

// Use the library modules
use nsfw::cli;
use nsfw::config::Config;
use nsfw::logging;

#[derive(Parser)]
#[command(name = "nsfw")]
#[command(author = "Luminous Dynamics")]
#[command(version = "0.3.0")]
#[command(about = "Nix Subsystem for Windows - Natural language Nix package management", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging (DEBUG level)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable logging to file (~/.nsfw/logs/nsfw.log)
    #[arg(long, global = true)]
    log_file: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for packages in nixpkgs
    #[command(alias = "find")]
    #[command(long_about = "Search for packages in the nixpkgs repository using fuzzy matching.

EXAMPLES:
    # Search for Firefox
    nsfw search firefox

    # Search with exact matching (no fuzzy)
    nsfw search --no-fuzzy python

    # Limit results to 10
    nsfw search nodejs --limit 10

    # Output as JSON
    nsfw search rust --format json

    # Search is case-insensitive and matches package names and descriptions
    nsfw search 'text editor'")]
    Search {
        /// Search query (package name or description)
        query: String,

        /// Maximum number of results to show
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Use fuzzy matching for search (default: true)
        #[arg(long, default_value_t = true)]
        fuzzy: bool,
    },

    /// Install one or more packages
    #[command(alias = "add")]
    #[command(long_about = "Install one or more packages from nixpkgs.

Packages are installed into your WSL2/Nix environment and wrapper scripts
are automatically generated in Windows for easy access.

EXAMPLES:
    # Install a single package
    nsfw install firefox

    # Install multiple packages at once
    nsfw install python3 nodejs git

    # Skip confirmation prompt
    nsfw install vim --yes

    # Preview what would be installed (dry-run)
    nsfw install chromium --dry-run

    # Alias: 'add' works the same as 'install'
    nsfw add rust cargo")]
    Install {
        /// Package name(s) (e.g., firefox, python3, nodejs)
        #[arg(required = true)]
        packages: Vec<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Show what would be installed without actually installing
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove one or more installed packages
    #[command(alias = "uninstall")]
    #[command(long_about = "Remove one or more installed packages from your system.

The packages will be uninstalled from your Nix profile and the corresponding
Windows wrapper scripts will be cleaned up.

EXAMPLES:
    # Remove a single package
    nsfw remove firefox

    # Remove multiple packages at once
    nsfw remove python3 nodejs git

    # Skip confirmation prompt
    nsfw remove vim --yes

    # Preview what would be removed (dry-run)
    nsfw remove chromium --dry-run

    # Alias: 'uninstall' works the same as 'remove'
    nsfw uninstall rust cargo")]
    Remove {
        /// Package name(s) to remove
        #[arg(required = true)]
        packages: Vec<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Show what would be removed without actually removing
        #[arg(long)]
        dry_run: bool,
    },

    /// List installed packages
    #[command(alias = "ls")]
    #[command(long_about = "List all packages currently installed via NSFW.

EXAMPLES:
    # List all installed packages
    nsfw list

    # Show detailed information for each package
    nsfw list --detailed

    # Output as JSON for scripting
    nsfw list --format json

    # Alias: 'ls' works the same as 'list'
    nsfw ls -d")]
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show information about a package
    #[command(long_about = "Display detailed information about a specific package.

Shows the package description, version, homepage, license, and other metadata.

EXAMPLES:
    # Get info about Firefox
    nsfw info firefox

    # Get info about Python 3
    nsfw info python3")]
    Info {
        /// Package name
        package: String,
    },

    /// Update the package database
    #[command(long_about = "Update the Nix channels to get the latest package listings.

This downloads the latest package database from NixOS and refreshes your
local cache. Run this periodically to access new packages and updates.

EXAMPLES:
    # Update the package database
    nsfw update

TIP: Run this command weekly to stay up-to-date with the latest packages.")]
    Update {},

    /// Setup WSL2 and Nix environment (first-time setup)
    #[command(long_about = "Run the initial setup wizard for NSFW.

This command guides you through installing WSL2, setting up Nix, and
configuring your environment. Only needs to be run once.

EXAMPLES:
    # Run interactive setup wizard
    nsfw setup

    # Skip all confirmation prompts
    nsfw setup --yes

    # Use interactive mode with options
    nsfw setup --interactive")]
    Setup {
        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,

        /// Use interactive mode to choose options
        #[arg(short, long)]
        interactive: bool,
    },

    /// Generate wrapper script for a package (internal)
    #[command(hide = true)]
    GenerateWrapper {
        /// Package name
        package: String,

        /// Package path
        package_path: String,
    },

    /// Manage package cache
    #[command(long_about = "Manage the local package cache database.

The cache stores package information locally for instant search results.

SUBCOMMANDS:
    stats    - Show cache statistics and information
    clear    - Clear the cache database
    rebuild  - Rebuild the cache from nixpkgs

EXAMPLES:
    # Show cache statistics
    nsfw cache stats

    # Clear the cache
    nsfw cache clear

    # Rebuild the cache
    nsfw cache rebuild")]
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },

    /// Diagnose system health and configuration
    #[command(long_about = "Run diagnostics to check your NSFW installation.

Checks WSL2 availability, Nix installation, package cache, permissions,
and disk space. Provides suggestions for any issues found.

EXAMPLES:
    # Run system diagnostics
    nsfw doctor

TIP: Run this if you're experiencing issues with NSFW.")]
    Doctor,

    /// View installation history
    #[command(long_about = "View the history of package installations and removals.

Tracks all package operations including successful installs, removals,
and any failures. Useful for debugging and auditing package changes.

EXAMPLES:
    # View recent history (last 20 operations)
    nsfw history

    # View last 50 operations
    nsfw history --limit 50

    # View history for a specific package
    nsfw history --package firefox

    # Show detailed statistics
    nsfw history --stats")]
    History {
        /// Maximum number of entries to show
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// Show history for a specific package
        #[arg(short, long)]
        package: Option<String>,

        /// Show statistics instead of history entries
        #[arg(short, long)]
        stats: bool,
    },

    /// Install shell completions
    #[command(long_about = "Install tab completion for your shell.

Enables intelligent tab completion for commands, packages, and options.

SUPPORTED SHELLS:
    - powershell (default)
    - bash
    - zsh
    - fish

EXAMPLES:
    # Install PowerShell completions
    nsfw completion

    # Install Bash completions
    nsfw completion bash

    # Install Zsh completions
    nsfw completion zsh")]
    Completion {
        /// Shell type (powershell, bash, zsh, fish)
        #[arg(default_value = "powershell")]
        shell: String,
    },

    /// Manage configuration settings
    #[command(long_about = "Manage NSFW configuration settings.

Configure behavior, output formatting, caching, and other options.

SUBCOMMANDS:
    show   - Display all configuration settings
    get    - Get a specific configuration value
    set    - Set a configuration value
    reset  - Reset configuration to defaults
    path   - Show configuration file location
    keys   - List all available configuration keys

EXAMPLES:
    # Show all settings
    nsfw config show

    # Get a specific setting
    nsfw config get disable_colors

    # Set a configuration value
    nsfw config set verbose_output true

    # List available keys
    nsfw config keys")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Upgrade installed package(s) to latest version
    #[command(long_about = "Upgrade one or all installed packages to their latest versions.

This removes the old version and installs the latest version from nixpkgs.

EXAMPLES:
    # Upgrade a specific package
    nsfw upgrade firefox

    # Upgrade all installed packages
    nsfw upgrade

    # Skip confirmation prompt
    nsfw upgrade --yes

    # Preview what would be upgraded (dry-run)
    nsfw upgrade --dry-run

TIP: Run 'nsfw update' before upgrading to ensure you get the latest versions.")]
    Upgrade {
        /// Package name (if omitted, upgrades all packages)
        package: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Show what would be upgraded without actually upgrading
        #[arg(long)]
        dry_run: bool,
    },

    /// Export installed packages to a file
    #[command(long_about = "Export your installed packages to a file for backup or sharing.

Creates a portable file containing all your installed packages, which can
be imported on another machine or used for backup/restore.

EXAMPLES:
    # Export to default file (nsfw-packages.json)
    nsfw export

    # Export to custom file
    nsfw export --output my-packages.json

    # Export as TOML
    nsfw export --format toml --output packages.toml

TIP: Commit this file to version control to reproduce your setup.")]
    Export {
        /// Output file path (defaults to nsfw-packages.json)
        #[arg(short, long, default_value = "nsfw-packages.json")]
        output: String,

        /// Output format (json, toml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Import and install packages from a file
    #[command(long_about = "Import and install packages from an exported package file.

Reads a package list file (JSON or TOML) and installs all packages listed.
Useful for setting up a new machine or restoring from backup.

EXAMPLES:
    # Import from a file
    nsfw import nsfw-packages.json

    # Skip confirmation prompt
    nsfw import packages.json --yes

    # Preview what would be imported (dry-run)
    nsfw import packages.json --dry-run

TIP: Packages already installed will be skipped automatically.")]
    Import {
        /// Input file path
        file: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Show what would be imported without actually installing
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show all configuration settings
    Show,

    /// Get a specific configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// New value
        value: String,
    },

    /// Reset configuration to defaults
    Reset,

    /// Show configuration file path
    Path,

    /// List all available configuration keys
    Keys,
}

#[derive(Subcommand)]
enum CacheAction {
    /// Show cache statistics
    Stats,

    /// Clear the package cache
    Clear,

    /// Rebuild the package cache
    Rebuild,
}

fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Load user configuration to get disable_colors setting
    let config = Config::load().unwrap_or_default();

    // Initialize custom logging system
    let log_config = logging::create_config(
        cli.verbose || config.verbose_output,
        cli.log_file,
        config.disable_colors,
    );
    logging::init(log_config);

    // Log startup message
    if cli.verbose || config.verbose_output {
        logging::debug("NSFW v0.3.0 starting...");
        logging::debug("Verbose mode: enabled");
        logging::debug(&format!("Log to file: {}", cli.log_file));
        logging::debug(&format!("Disable colors: {}", config.disable_colors));
    }

    // Execute command
    let result = match cli.command {
        Commands::Search { query, limit, format, fuzzy } => {
            cli::commands::search(&query, limit, &format, fuzzy)
        }
        Commands::Install { packages, yes, dry_run } => {
            cli::commands::install_batch(&packages, yes, dry_run)
        }
        Commands::Remove { packages, yes, dry_run } => {
            cli::commands::remove_batch(&packages, yes, dry_run)
        }
        Commands::List { detailed, format } => {
            cli::commands::list(detailed, &format)
        }
        Commands::Info { package } => {
            cli::commands::info(&package)
        }
        Commands::Update {} => {
            cli::commands::update()
        }
        Commands::Setup { yes, interactive } => {
            cli::commands::setup(yes, interactive)
        }
        Commands::GenerateWrapper { package, package_path } => {
            cli::commands::generate_wrapper(&package, &package_path)
        }
        Commands::Cache { action } => {
            match action {
                CacheAction::Stats => cli::commands::cache_stats(),
                CacheAction::Clear => cli::commands::cache_clear(),
                CacheAction::Rebuild => cli::commands::cache_rebuild(),
            }
        }
        Commands::Doctor => {
            cli::commands::doctor()
        }
        Commands::History { limit, package, stats } => {
            cli::commands::history(limit, package.as_deref(), stats)
        }
        Commands::Completion { shell } => {
            cli::commands::install_completion(&shell)
        }
        Commands::Config { action } => {
            match action {
                ConfigAction::Show => cli::commands::config_show(),
                ConfigAction::Get { key } => cli::commands::config_get(&key),
                ConfigAction::Set { key, value } => cli::commands::config_set(&key, &value),
                ConfigAction::Reset => cli::commands::config_reset(),
                ConfigAction::Path => cli::commands::config_path(),
                ConfigAction::Keys => cli::commands::config_keys(),
            }
        }
        Commands::Upgrade { package, yes, dry_run } => {
            cli::commands::upgrade(package.as_deref(), yes, dry_run)
        }
        Commands::Export { output, format } => {
            cli::commands::export(&output, &format)
        }
        Commands::Import { file, yes, dry_run } => {
            cli::commands::import(&file, yes, dry_run)
        }
    };

    // Handle errors
    if let Err(e) = result {
        logging::error(&format!("Command failed: {}", e));
        std::process::exit(1);
    }
}
