//! # CLI Argument Definitions
//!
//! This module defines the command-line interface (CLI) structure using the `clap` crate.
//! It specifies the available subcommands, arguments, and flags for the application.

use clap::{Parser, Subcommand};

/// The main CLI structure parsing command-line arguments.
#[derive(Parser)]
#[command(name = "cargo xtask")]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// The main subcommand to execute.
    #[command(subcommand)]
    pub command: AppCommands,
}

/// Enumeration of available application subcommands.
#[derive(Subcommand)]
pub enum AppCommands {
    /// Install the necessary tools and dependencies for development
    Setup {},
    /// Manage workspace Features
    Features {
        #[command(subcommand)]
        action: FeatureAction,
    },
    /// Manage workspace Infrastructure
    Libs {
        #[command(subcommand)]
        action: LibraryAction,
    },
    /// Manage workspace Applications
    Apps {
        #[command(subcommand)]
        action: AppAction,
    },
    /// Manage local development infrastructure
    Dev {
        #[command(subcommand)]
        action: DevAction,
    },
    /// Run all workspace tests and lints
    Test {},
    /// Universal License Generator
    License {
        #[arg(short, long)]
        customer: String,

        /// Comma-separated list of IDs, or use 'ANY' for site license
        #[arg(short, long, default_value = "ANY")]
        machines: String,

        /// Comma-separated features (e.g. 'quiz,survey, pass' or '*' for all features)
        #[arg(short, long, default_value = "*")]
        features: String,

        #[arg(short, long, default_value_t = 365)]
        days: u64,
    },
}

#[derive(Subcommand)]
pub enum FeatureAction {
    /// Create a new feature in the crates/ directory
    Add {
        /// The name of the feature (will be prefixed with 'mhub-')
        name: String,
    },
    /// List all features in the crates/ directory with their descriptions
    List {},
}

#[derive(Subcommand)]
pub enum LibraryAction {
    /// Create a new library in the infra/ directory
    Add {
        /// The name of the library (will be prefixed with 'mhub-')
        name: String,
    },
    /// List all libraries in the infra/ directory with their descriptions
    List {},
}

#[derive(Subcommand)]
pub enum AppAction {
    /// Create a new application in the apps/ directory
    Add {
        /// The name of the application (will be prefixed with 'mhub-')
        name: String,
    },
    /// List all applications in the apps/ directory with their descriptions
    List {},
}

/// Enumeration of available development subcommands.
#[derive(Subcommand)]
pub enum DevAction {
    /// Start SurrealDB and other services
    Up {},
    /// Stop all services
    Down {
        /// Also remove volumes (wipes the database)
        #[arg(short, long)]
        volumes: bool,
    },
    /// Follow logs from services
    Logs {
        /// Specific service name (e.g., 'surrealdb')
        service: Option<String>,
    },
}
