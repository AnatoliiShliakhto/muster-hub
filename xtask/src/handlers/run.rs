use crate::services::utils::normalize_project_name;
use anyhow::{Context, bail};

/// Runs a project with `cargo run`.
///
/// # Result
/// Returns an `anyhow::Result<()>` indicating success or failure of the run.
///
/// # Errors
/// Returns an error if the project fails to build or exits with a non-zero status.
pub fn run_project(project: &str) -> anyhow::Result<()> {
    println!("🚀 Starting project...");

    let project = normalize_project_name(project);
    let status = std::process::Command::new("cargo")
        .args(["run", "-p", &project])
        .status()
        .context("Failed to execute cargo run")?;

    if !status.success() {
        bail!("Project exited with non-zero status: {}", status.code().unwrap_or(-1));
    }

    Ok(())
}
