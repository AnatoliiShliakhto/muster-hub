use crate::services::utils::normalize_project_name;
use anyhow::{Context, bail};

/// Runs project with tokio-console and dhat profiling enabled.
///
/// This command compiles and runs the server with:
/// * `tokio_unstable` cfg flag enabled (required for tokio-console)
/// * `profiling` feature enabled
/// * Real-time async runtime diagnostics via tokio-console
///
/// # Result
/// Returns an `anyhow::Result<()>` indicating success or failure of the profiling run.
///
/// # Errors
/// Returns an error if:
/// * The server fails to build
/// * The server crashes or exits with a non-zero status code
pub fn run_profiling(project: &str) -> anyhow::Result<()> {
    println!("📊 Starting server with profiling...");
    println!("💡 Tip: Connect to http://localhost:6669 with the tokio-console CLI");

    let project = normalize_project_name(project);
    let status = std::process::Command::new("cargo")
        .env("RUSTFLAGS", "--cfg tokio_unstable")
        .args(["run", "-p", &project, "--features", "profiling"])
        .status()
        .context("Failed to execute cargo run")?;

    if !status.success() {
        bail!("Server exited with non-zero status: {}", status.code().unwrap_or(-1));
    }

    println!("✅ Profiling completed successfully");
    Ok(())
}
