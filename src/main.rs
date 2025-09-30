use clap::{Parser, Subcommand};
use log::{info, error};

mod cli;
mod nix_ops;
mod path_translation;
mod templates;
mod wsl_bridge;

#[derive(Parser)]
#[command(name = "nsfw")]
#[command(author = "Luminous Dynamics")]
#[command(version = "0.1.0")]
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

    /// Generate wrapper script for a package (internal)
    #[command(hide = true)]
    GenerateWrapper {
        /// Package name
        package: String,

        /// Package path
        package_path: String,
    },
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

    info!("NSFW v0.1.0 starting...");

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
        Commands::GenerateWrapper { package, package_path } => {
            cli::commands::generate_wrapper(&package, &package_path)
        }
    };

    // Handle errors
    if let Err(e) = result {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }
}
