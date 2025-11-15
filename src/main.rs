use clap::{Parser, Subcommand};
use log::{info, error};

// Use the library modules
use nsfw::cli;

#[derive(Parser)]
#[command(name = "nsfw")]
#[command(author = "Luminous Dynamics")]
#[command(version = "0.3.0")]
#[command(about = "Nix Subsystem for Windows - Natural language Nix package management", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for packages in nixpkgs
    #[command(alias = "find")]
    Search {
        /// Search query (package name or description)
        query: String,

        /// Maximum number of results to show
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Install a package
    #[command(alias = "add")]
    Install {
        /// Package name (e.g., firefox, python3)
        package: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Remove an installed package
    #[command(alias = "uninstall")]
    Remove {
        /// Package name to remove
        package: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// List installed packages
    #[command(alias = "ls")]
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show information about a package
    Info {
        /// Package name
        package: String,
    },

    /// Update the package database
    Update {},

    /// Setup WSL2 and Nix environment (first-time setup)
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
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },

    /// Diagnose system health and configuration
    Doctor,

    /// Install shell completions
    Completion {
        /// Shell type (powershell, bash, zsh, fish)
        #[arg(default_value = "powershell")]
        shell: String,
    },

    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Upgrade installed package(s) to latest version
    Upgrade {
        /// Package name (if omitted, upgrades all packages)
        package: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Export installed packages to a file
    Export {
        /// Output file path (defaults to nsfw-packages.json)
        #[arg(short, long, default_value = "nsfw-packages.json")]
        output: String,

        /// Output format (json, toml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Import and install packages from a file
    Import {
        /// Input file path
        file: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
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

    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    info!("NSFW v0.3.0 starting...");

    // Execute command
    let result = match cli.command {
        Commands::Search { query, limit, format } => {
            cli::commands::search(&query, limit, &format)
        }
        Commands::Install { package, yes } => {
            cli::commands::install(&package, yes)
        }
        Commands::Remove { package, yes } => {
            cli::commands::remove(&package, yes)
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
        Commands::Upgrade { package, yes } => {
            cli::commands::upgrade(package.as_deref(), yes)
        }
        Commands::Export { output, format } => {
            cli::commands::export(&output, &format)
        }
        Commands::Import { file, yes } => {
            cli::commands::import(&file, yes)
        }
    };

    // Handle errors
    if let Err(e) = result {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }
}
