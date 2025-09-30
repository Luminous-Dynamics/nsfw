//! NSFW - Nix Subsystem for Windows
//!
//! A natural language interface for NixOS package management on Windows via WSL2.
//!
//! This library provides the core functionality for bridging Windows and WSL2/Nix,
//! including path translation, WSL2 command execution, and Nix operations.

// Re-export public modules
pub mod nix_ops;
pub mod path_translation;
pub mod templates;
pub mod wsl2;

// Re-export CLI module for internal use
pub mod cli;
