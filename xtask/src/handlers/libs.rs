use crate::error::AppError;
use crate::services::cargo::register_in_workspace;
use crate::services::utils::{
    get_project_root, get_workspace_crates, refresh_metadata, render_crate_table,
};
use cargo_generate::{GenerateArgs, TemplatePath, generate};
use heck::ToKebabCase;

/// Lists all libraries in the `infra/` directory.
///
/// # Result
/// Returns `Ok(())` after printing the library table (or a friendly empty-state message).
///
/// # Errors
/// Returns an error if the directory cannot be read or crate metadata cannot be parsed.
pub fn list_libs() -> Result<(), AppError> {
    let libraries = get_workspace_crates("infra")?;

    if libraries.is_empty() {
        println!("ℹ️ No libs found in `infra/` directory.");
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
pub fn create_lib(name: &str) -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let full_package_name = format!("mhub-{name}").to_kebab_case();
    let define = vec![format!("name=mhub-{name}"), format!("shortname={name}")];

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

    generate(args).map_err(|e| {
        AppError::codegen().with_context_fn(|| format!("Failed to generate infra crate: {e}"))
    })?;
    register_in_workspace(&project_root, &full_package_name, &format!("infra/{name}"))?;
    refresh_metadata()?;

    println!("✅ Created lib `mhub-{name}` with package `infra/{name}`");
    Ok(())
}
