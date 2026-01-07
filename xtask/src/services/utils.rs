use crate::error::{AppError, AppErrorExt};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::de;

/// Returns the root directory of the project.
///
/// # Result
/// Returns the workspace root path as `PathBuf`.
///
/// # Errors
/// Returns an error if the manifest directory does not have a parent.
pub fn get_project_root() -> Result<PathBuf, AppError> {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().map(Path::to_path_buf).ok_or_else(|| {
        AppError::internal()
            .with_message("Failed to resolve project root")
            .with_context_fn(|| format!("Manifest dir was `{}`", env!("CARGO_MANIFEST_DIR")))
            .with_help("The xtask crate must be located in a sub-directory of the workspace root")
    })
}

#[derive(Debug, Deserialize)]
pub struct CrateInfo {
    #[serde(skip)]
    pub path: PathBuf,
    pub package: PackageInfo,
}

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: Option<String>,
}

/// Discovers crates in a workspace subdirectory (e.g., "crates/features", "apps", "infra").
///
/// # Result
/// Returns a list of discovered crates, each with parsed package metadata.
///
/// # Errors
/// Returns an error if the directory cannot be read, a `Cargo.toml` cannot be read,
/// or the metadata cannot be parsed.
pub fn get_workspace_crates(sub_dir: &str) -> Result<Vec<CrateInfo>, AppError> {
    let project_root = get_project_root()?;
    let target_dir = project_root.join(sub_dir);
    let mut crates = Vec::new();

    if !target_dir.exists() {
        return Ok(crates);
    }

    for entry in fs::read_dir(&target_dir)
        .with_context_fn(|| format!("Failed to read directory `{}`", target_dir.display()))?
    {
        let entry = entry.with_context_fn(|| {
            format!("Failed to read entry in directory `{}`", target_dir.display())
        })?; // Simplified error mapping
        let path = entry.path();
        let cargo_path = path.join("Cargo.toml");

        if path.is_dir() && cargo_path.exists() {
            let content = fs::read_to_string(&cargo_path).with_context_fn(|| {
                format!("Failed to read Cargo.toml at `{}`", cargo_path.display())
            })?;

            let mut info: CrateInfo = de::from_str(&content).map_err(|e| {
                AppError::parse().with_message(e.to_string()).with_context_fn(|| {
                    format!("Failed to parse Cargo.toml at `{}`", cargo_path.display())
                })
            })?;

            info.path = path;
            crates.push(info);
        }
    }

    // Sort by folder name
    crates.sort_by_key(|c| c.path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_owned());

    Ok(crates)
}

/// Prints a formatted table of crates with their folder, name, and description.
///
/// # Result
/// Returns `()` after printing the table.
///
/// # Errors
/// This function does not return errors.
pub fn render_crate_table(title: &str, crates: &[CrateInfo]) {
    println!("\n{title}:\n");
    println!("{:<15} {:<20} {:<45}", "Folder", "Crate Name", "Description");
    println!("{:-<80}", "");

    for info in crates {
        let folder = info.path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        let desc = info.package.description.as_deref().unwrap_or("No description provided");

        println!("{:<15} {:<20} {:<45}", folder, info.package.name, desc);
    }
    println!();
}

/// Normalizes a project crate name to the workspace naming convention.
#[must_use]
pub fn normalize_project_name(project: &str) -> String {
    if project.starts_with("mhub-") { project.to_owned() } else { format!("mhub-{project}") }
}

/// Refreshes workspace metadata to keep tooling in sync.
///
/// # Result
/// Returns `Ok(())` after running `cargo metadata`.
///
/// # Errors
/// Returns an error if `cargo metadata` fails to run or exits unsuccessfully.
pub fn refresh_metadata() -> Result<(), AppError> {
    println!("Refreshing workspace metadata...");
    let status = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .stdout(std::process::Stdio::null())
        .status()
        .with_context("Failed to execute cargo command")
        .with_help("Is Cargo installed and accessible in your PATH?")?;

    if !status.success() {
        Err(AppError::internal().with_context_fn(|| {
            format!("Cargo exited with status: {}", status.code().unwrap_or(-1))
        }))?;
    }

    Ok(())
}
