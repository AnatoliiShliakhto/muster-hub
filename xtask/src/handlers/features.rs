use crate::services::utils::get_project_root;
use anyhow::Result;
use cargo_generate::{GenerateArgs, TemplatePath, generate};

pub fn create_feature(name: &str) -> Result<()> {
    let project_root = get_project_root()?;

    let define = vec![
        format!("package_name=muster-{name}"),
        format!("package_description=It's a new feature muster-{name}"),
    ];

    let args = GenerateArgs {
        name: Some(name.to_owned()),
        destination: Some(project_root.join("crates")),
        define,
        template_path: TemplatePath {
            path: Some("xtask/templates/feature-template".to_owned()),
            ..Default::default()
        },
        silent: true,
        ..Default::default()
    };

    generate(args)?;
    refresh_metadata()?;

    println!("✅ Created crate '{name}' with package 'muster-{name}'");
    Ok(())
}

fn refresh_metadata() -> Result<()> {
    println!("Refreshing workspace metadata...");
    std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .stdout(std::process::Stdio::null())
        .status()?;
    Ok(())
}
