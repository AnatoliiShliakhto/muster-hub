use crate::error::{AppError, AppErrorExt};
use std::fs;

/// Registers a new package in the workspace's Cargo.toml file.
///
/// # Result
///
/// # Errors
///
pub fn register_in_workspace(
    root: &std::path::Path,
    pkg_name: &str,
    pkg_path: &str,
) -> Result<(), AppError> {
    let toml_path = root.join("Cargo.toml");
    let content = fs::read_to_string(&toml_path).with_context("Failed to read root Cargo.toml")?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| AppError::parse().with_message(format!("Failed to parse Cargo.toml: {e}")))?;

    if let Some(deps) = doc
        .get_mut("workspace")
        .and_then(|w| w.as_table_mut())
        .and_then(|w| w.get_mut("dependencies"))
        .and_then(|d| d.as_table_mut())
    {
        let mut inline_table = toml_edit::InlineTable::default();
        inline_table.insert("path", pkg_path.into());
        deps.insert(pkg_name, toml_edit::value(inline_table));

        deps.sort_values();
    }

    fs::write(&toml_path, doc.to_string()).with_context("Failed to update root Cargo.toml")?;
    Ok(())
}
