#![warn(rust_2018_idioms, unused_lifetimes)]
#![allow(
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

pub mod handlers;
pub mod models;
pub mod services;

use crate::handlers::{apps, bench, codegen, dev, features, libs, profiling, run, setup, testing};
use crate::models::args::{
    AppAction, AppCommands, Cli, CodegenAction, FeatureAction, LibraryAction,
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
        AppCommands::Codegen { action } => match action {
            CodegenAction::Migrations {} => codegen::codegen_migrations()?,
        },
        AppCommands::Test { project } => testing::run_tests(project.as_deref())?,
        AppCommands::Doctest { project } => testing::run_doctests(project.as_deref())?,
        AppCommands::Run { project } => run::run_project(&project)?,
        AppCommands::Bench { project } => bench::run_bench(&project)?,
        AppCommands::Profiling { project } => profiling::run_profiling(&project)?,
        AppCommands::Lic { customer, alias, machines, matches, features, days } => {
            handlers::license::generate_license(
                &customer, &alias, &machines, matches, &features, days,
            )?;
        },
    }

    Ok(())
}
