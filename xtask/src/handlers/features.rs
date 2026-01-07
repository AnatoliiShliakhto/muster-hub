use crate::services::utils::{
    get_project_root, get_workspace_crates, refresh_metadata, render_crate_table,
};
use anyhow::Result;
use cargo_generate::{GenerateArgs, TemplatePath, generate};

/// Lists all crates in the `crates/features` directory.
///
/// # Result
/// Returns `Ok(())` after printing the feature table (or a friendly empty-state message).
///
/// # Errors
/// Returns an error if the directory cannot be read or crate metadata cannot be parsed.
pub fn list_crates() -> Result<()> {
    let features = get_workspace_crates("crates/features")?;

    if features.is_empty() {
        println!("ℹ️ No features found in 'crates/features/' directory.");
        return Ok(());
    }

    render_crate_table("Features", &features);

    Ok(())
}

/// Creates a feature crate from the template.
///
/// # Result
/// Returns `Ok(())` after scaffolding the crate and refreshing workspace metadata.
///
/// # Errors
/// Returns an error if the template generation fails, the destination cannot be
/// written, or the workspace metadata refresh fails.
pub fn create_feature(name: &str) -> Result<()> {
    let project_root = get_project_root()?;

    let define = vec![
        format!("package_name=mhub-{name}"),
        format!("package_description=It's a new feature {name}"),
    ];

    let args = GenerateArgs {
        name: Some(name.to_owned()),
        destination: Some(project_root.join("crates").join("features")),
        define,
        template_path: TemplatePath {
            path: Some("xtask/templates/feature".to_owned()),
            ..Default::default()
        },
        silent: true,
        ..Default::default()
    };

    generate(args)?;
    refresh_metadata()?;

    println!("✅ Created feature 'mhub-{name}' with package 'crates/features/{name}'");
    Ok(())
}
