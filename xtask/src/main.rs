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

use crate::handlers::{apps, dev, features, libs, setup, testing};
use crate::models::args::{
    AppAction, AppCommands, Cli, FeatureAction, LibraryAction,
};

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        AppCommands::Setup {} => setup::setup_project()?,
        AppCommands::Features { action } => match action {
            FeatureAction::Add { name } => features::create_feature(&name)?,
            FeatureAction::List {} => features::list_crates()?,
        },
        AppCommands::Libs { action } => match action {
            LibraryAction::Add { name } => libs::create_lib(&name)?,
            LibraryAction::List {} => libs::list_libs()?,
        },
        AppCommands::Apps { action } => match action {
            AppAction::Add { name } => apps::create_app(&name)?,
            AppAction::List {} => apps::list_apps()?,
        },
        AppCommands::Dev { action } => dev::handle_dev_command(action)?,
        AppCommands::Test {} => testing::run_all_tests()?,
        AppCommands::License { customer, machines, features, days } => {
            handlers::license::generate_license(
                &customer, &machines, &features, days,
            )?;
        },
    }

    Ok(())
}
