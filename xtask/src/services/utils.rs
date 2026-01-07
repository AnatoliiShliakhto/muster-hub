use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn get_project_root() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .context("Could not find project root from xtask manifest")
}
