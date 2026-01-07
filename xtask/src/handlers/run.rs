use crate::error::{AppError, AppErrorExt};
use crate::services::utils::normalize_project_name;

/// Runs a project with `cargo run`.
///
/// # Result
/// Returns an `Ok(())` indicating success or failure of the run.
///
/// # Errors
/// Returns an error if the project fails to build or exits with a non-zero status.
pub fn run_project(project: &str) -> Result<(), AppError> {
    println!("🚀 Starting project...");

    let project = normalize_project_name(project);
    let status = std::process::Command::new("cargo")
        .args(["run", "-p", &project])
        .status()
        .with_context("Failed to execute cargo run")?;

    if !status.success() {
        Err(AppError::internal().with_context_fn(|| {
            format!("Project exited with status: {}", status.code().unwrap_or(-1))
        }))?;
    }

    Ok(())
}
