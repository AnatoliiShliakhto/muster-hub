use crate::services::utils::normalize_project_name;
use anyhow::{Context, bail};

/// Runs benches for a project with a ` cargo bench `.
///
/// # Result
/// Returns an `anyhow::Result<()>` indicating success or failure of the bench run.
///
/// # Errors
/// Returns an error if the bench build or run fails.
pub fn run_bench(project: &str) -> anyhow::Result<()> {
    println!("🏁 Running benches...");

    let project = normalize_project_name(project);
    let status = std::process::Command::new("cargo")
        .args(["bench", "-p", &project, "--all-features"])
        .status()
        .context("Failed to execute cargo bench")?;

    if !status.success() {
        bail!("Bench exited with non-zero status: {}", status.code().unwrap_or(-1));
    }

    Ok(())
}
