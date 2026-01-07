use crate::error::{AppError, AppErrorExt};
use crate::services::utils::normalize_project_name;

/// Runs benches for a project with a ` cargo bench `.
///
/// # Result
/// Returns an `Ok(())` indicating success or failure of the bench run.
///
/// # Errors
/// Returns an error if the bench build or run fails.
pub fn run_bench(project: &str) -> Result<(), AppError> {
    println!("🏁 Running benches...");

    let project = normalize_project_name(project);
    let status = std::process::Command::new("cargo")
        .args(["bench", "-p", &project, "--all-features"])
        .status()
        .with_context("Failed to execute cargo bench")
        .with_help("Is Cargo installed and accessible in your PATH?")?;

    if !status.success() {
        Err(AppError::internal().with_context_fn(|| {
            format!("Bench exited with status: {}", status.code().unwrap_or(-1))
        }))?;
    }

    Ok(())
}
