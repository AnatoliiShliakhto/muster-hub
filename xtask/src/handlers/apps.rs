use crate::error::AppError;
use crate::services::cargo::register_in_workspace;
use crate::services::utils::{
    get_project_root, get_workspace_crates, refresh_metadata, render_crate_table,
};
use cargo_generate::{GenerateArgs, TemplatePath, generate};
use heck::ToKebabCase;

/// Lists all apps in the `apps/` directory.
///
/// # Result
/// Returns `Ok(())` after printing the app table (or a friendly empty-state message).
///
/// # Errors
/// Returns an error if the `apps/` directory cannot be read or the crate metadata
/// cannot be parsed.
pub fn list_apps() -> Result<(), AppError> {
    let applications = get_workspace_crates("apps")?;

    if applications.is_empty() {
        println!("ℹ️ No apps found in `apps/` directory.");
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
pub fn create_app(name: &str) -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let full_package_name = format!("mhub-{name}").to_kebab_case();
    let define = vec![format!("name=mhub-{name}"), format!("shortname={name}")];

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

    generate(args).map_err(|e| {
        AppError::codegen()
            .with_message(e.to_string())
            .with_context_fn(|| format!("Failed to generate app crate `{name}`"))
    })?;
    register_in_workspace(&project_root, &full_package_name, &format!("apps/{name}"))?;
    refresh_metadata()?;

    println!("✅ Created app `mhub-{name}` with package `apps/{name}`");
    Ok(())
}
