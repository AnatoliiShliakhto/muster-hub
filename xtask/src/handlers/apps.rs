use crate::services::utils::{
    get_project_root, get_workspace_crates, refresh_metadata, render_crate_table,
};
use anyhow::Result;
use cargo_generate::{GenerateArgs, TemplatePath, generate};

/// Lists all apps in the `apps/` directory.
///
/// # Result
/// Returns `Ok(())` after printing the app table (or a friendly empty-state message).
///
/// # Errors
/// Returns an error if the `apps/` directory cannot be read or the crate metadata
/// cannot be parsed.
pub fn list_apps() -> Result<()> {
    let applications = get_workspace_crates("apps")?;

    if applications.is_empty() {
        println!("ℹ️ No apps found in 'apps/' directory.");
        return Ok(());
    }

    render_crate_table("Applications", &applications);

    Ok(())
}

/// Creates an app crate from the template.
///
/// # Result
/// Returns `Ok(())` after scaffolding the app and refreshing workspace metadata.
///
/// # Errors
/// Returns an error if the template generation fails, the destination cannot be
/// written, or the workspace metadata refresh fails.
pub fn create_app(name: &str) -> Result<()> {
    let project_root = get_project_root()?;

    let define = vec![
        format!("bin_name=mhub-{name}"),
        format!("package_name=mhub-{name}"),
        format!("package_description=It's a new application {name}"),
    ];

    let args = GenerateArgs {
        name: Some(name.to_owned()),
        destination: Some(project_root.join("apps")),
        define,
        template_path: TemplatePath {
            path: Some("xtask/templates/app".to_owned()),
            ..Default::default()
        },
        silent: true,
        ..Default::default()
    };

    generate(args)?;
    refresh_metadata()?;

    println!("✅ Created app 'mhub-{name}' with package 'apps/{name}'");
    Ok(())
}
