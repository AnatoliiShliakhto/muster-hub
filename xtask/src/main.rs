#![warn(rust_2018_idioms, unused_lifetimes)]
#![allow(
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

mod handlers;
mod models;
mod services;

use crate::handlers::features::create_feature;
use crate::models::args::{AppCommands, Cli, FeatureAction};

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        AppCommands::Features { action } => match action {
            FeatureAction::Add { name } => create_feature(&name)?,
        },
    }

    Ok(())
}
