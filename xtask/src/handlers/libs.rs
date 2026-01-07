use crate::services::utils::{
    get_project_root, get_workspace_crates, refresh_metadata, render_crate_table,
};
use anyhow::Result;
use cargo_generate::{GenerateArgs, TemplatePath, generate};

/// Lists all libraries in the `infra/` directory.
///
/// # Result
/// Returns `Ok(())` after printing the library table (or a friendly empty-state message).
///
/// # Errors
/// Returns an error if the directory cannot be read or crate metadata cannot be parsed.
pub fn list_libs() -> Result<()> {
    let libraries = get_workspace_crates("infra")?;

    if libraries.is_empty() {
        println!("ℹ️ No libs found in 'infra/' directory.");
        return Ok(());
    }

    render_crate_table("Infrastructure", &libraries);

    Ok(())
}

/// Creates an infrastructure crate from the template.
///
/// # Result
/// Returns `Ok(())` after scaffolding the crate and refreshing workspace metadata.
///
/// # Errors
/// Returns an error if the template generation fails, the destination cannot be
/// written, or the workspace metadata refresh fails.
pub fn create_lib(name: &str) -> Result<()> {
    let project_root = get_project_root()?;

    let define = vec![
        format!("package_name=mhub-{name}"),
        format!("package_description=It's a new library mhub-{name}"),
    ];

    let args = GenerateArgs {
        name: Some(name.to_owned()),
        destination: Some(project_root.join("infra")),
        define,
        template_path: TemplatePath {
            path: Some("xtask/templates/lib".to_owned()),
            ..Default::default()
        },
        silent: true,
        ..Default::default()
    };

    generate(args)?;
    refresh_metadata()?;

    println!("✅ Created lib 'mhub-{name}' with package 'infra/{name}'");
    Ok(())
}
