use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_project_root() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .context("Could not find project root from xtask manifest")
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

/// Discovers crates in a specific workspace directory (e.g., "crates", "apps", "infra")
pub fn get_workspace_crates(sub_dir: &str) -> Result<Vec<CrateInfo>> {
    let project_root = get_project_root()?;
    let target_dir = project_root.join(sub_dir);

    let mut crates = Vec::new();

    if !target_dir.exists() {
        return Ok(crates);
    }

    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();
        let cargo_path = path.join("Cargo.toml");

        if path.is_dir() && cargo_path.exists() {
            let content = fs::read_to_string(&cargo_path)?;
            let mut info: CrateInfo = toml::from_str(&content)?;
            info.path = path;
            crates.push(info);
        }
    }

    Ok(crates)
}

pub fn render_crate_table(title: &str, crates: &[CrateInfo]) {
    println!("\n{title}:\n");
    println!("{:<15} {:<20} {:<45}", "Folder", "Crate Name", "Description");
    println!("{:-<80}", "");

    for info in crates {
        let folder =
            info.path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        let desc = info
            .package
            .description
            .as_deref()
            .unwrap_or("No description provided");

        println!("{:<15} {:<20} {:<45}", folder, info.package.name, desc);
    }
    println!();
}

pub fn refresh_metadata() -> Result<()> {
    println!("Refreshing workspace metadata...");
    std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .stdout(std::process::Stdio::null())
        .status()?;
    Ok(())
}
