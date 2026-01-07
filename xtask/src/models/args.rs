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
    /// Manage workspace crates/features
    Features {
        #[command(subcommand)]
        action: FeatureAction,
    },
}

#[derive(Subcommand)]
pub enum FeatureAction {
    /// Create a new crate in the crates/ directory
    Add {
        /// The name of the feature (will be prefixed with 'muster-')
        name: String,
    },
}
