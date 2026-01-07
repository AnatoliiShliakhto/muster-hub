//! # CLI Argument Definitions
//!
//! This module defines the command-line interface (CLI) structure using the `clap` crate.
//! It specifies the available subcommands, arguments, and flags for the application.

use clap::{Parser, Subcommand};

/// The main CLI structure parsing command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "cargo xtask")]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(arg_required_else_help = true)]
#[command(about = "Developer toolkit for the MusterHub workspace")]
pub struct Cli {
    /// The main subcommand to execute.
    #[command(subcommand)]
    pub command: AppCommands,
}

/// Enumeration of available application subcommands.
#[derive(Debug, Subcommand)]
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
    /// Generate code artifacts
    #[command(alias = "migrations")]
    Codegen {
        #[command(subcommand)]
        action: CodegenAction,
    },
    /// Run tests (workspace by default)
    Test {
        /// Run tests for a specific crate (auto-prefixes with 'mhub-' if missing)
        project: Option<String>,
    },
    /// Run doc tests (workspace by default)
    Doctest {
        /// Run doc tests for a specific crate (auto-prefixes with 'mhub-' if missing)
        project: Option<String>,
    },
    /// Run a project
    Run {
        /// Run a specific crate (auto-prefixes with 'mhub-' if missing)
        project: String,
    },
    /// Run benches for a project
    Bench {
        /// Run benches for a specific crate (auto-prefixes with 'mhub-' if missing)
        project: String,
    },
    /// Run a project with profiling (requires unstable tokio)
    Profiling {
        #[arg(short, long)]
        project: String,
    },
    /// Universal License Generator
    Lic {
        /// The name of the customer (e.g., '`MusterHub` Inc.')
        #[arg(short, long)]
        customer: String,

        /// Short alias for namespaces and resource naming.
        #[arg(short, long)]
        alias: String,

        /// Comma-separated list of IDs, or use 'ANY' for site license
        #[arg(short, long, default_value = "ANY")]
        machines: String,

        /// Minimum number of hardware matches required (for fuzzy matching)
        #[arg(long, default_value_t = 1)]
        matches: u16,

        /// Comma-separated features (e.g. 'quiz,survey, pass' or '*' for all features)
        #[arg(short, long, default_value = "*")]
        features: String,

        #[arg(short, long, default_value_t = 365)]
        days: u64,
    },
}

#[derive(Debug, Subcommand)]
pub enum FeatureAction {
    /// Create a new feature in the crates/ directory
    Add {
        /// The name of the feature (will be prefixed with 'mhub-')
        name: String,
    },
    /// List all features in the crates/ directory with their descriptions
    List {},
}

#[derive(Debug, Subcommand)]
pub enum LibraryAction {
    /// Create a new library in the infra/ directory
    Add {
        /// The name of the library (will be prefixed with 'mhub-')
        name: String,
    },
    /// List all libraries in the infra/ directory with their descriptions
    List {},
}

#[derive(Debug, Subcommand)]
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
#[derive(Debug, Subcommand)]
pub enum DevAction {
    /// Start `SurrealDB` and other services
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

/// Enumeration of codegen commands.
#[derive(Debug, Subcommand)]
pub enum CodegenAction {
    /// Generate a hardcoded migration manifest
    Migrations {},
}
