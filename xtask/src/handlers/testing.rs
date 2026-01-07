use crate::services::utils::normalize_project_name;

/// Runs tests in the workspace or a specific crate.
///
/// # Result
/// Returns an `anyhow::Result<()>` indicating success or failure of the test run.
///
/// # Errors
/// Returns an error if the test execution fails or if the test runner is not found.
pub fn run_tests(project: Option<&str>) -> anyhow::Result<()> {
    let target_is_workspace = project.map_or(true, |value| value == "all");
    let target_label = if target_is_workspace { "workspace" } else { "crate" };

    println!("🧪 Running {target_label} tests...");
    let has_nextest = std::process::Command::new("cargo-nextest").arg("--version").output().is_ok();

    let mut args: Vec<String> = if has_nextest {
        vec!["nextest", "run"].into_iter().map(String::from).collect()
    } else {
        vec!["test"].into_iter().map(String::from).collect()
    };

    if target_is_workspace {
        args.push("--workspace".into());
    } else if let Some(project) = project {
        let normalized = normalize_project_name(project);
        args.push("-p".into());
        args.push(normalized);
    }

    args.push("--all-features".into());

    if has_nextest {
        args.extend(
            [
                "--failure-output",
                "immediate-final",
                "--success-output",
                "never",
                "--status-level",
                "skip",
            ]
            .into_iter()
            .map(String::from),
        );
    } else {
        args.extend(["--tests", "--lib", "bins", "--", "-q"].into_iter().map(String::from));
    }

    println!("🧪 Running tests via '{}'...", if has_nextest { "nextest" } else { "cargo test" });
    let status = std::process::Command::new("cargo").args(args).status()?;

    if !status.success() {
        anyhow::bail!("Tests failed!");
    }
    Ok(())
}

/// Runs doc tests in the workspace or a specific crate.
///
/// # Result
/// Returns an `anyhow::Result<()>` indicating success or failure of the doctest run.
///
/// # Errors
/// Returns an error if:
/// * The doctest execution fails
/// * A project name is required but not provided
pub fn run_doctests(project: Option<&str>) -> anyhow::Result<()> {
    let target_is_workspace = project.map_or(true, |value| value == "all");
    let target_label = if target_is_workspace { "workspace" } else { "crate" };

    println!("📚 Running {target_label} doc tests...");

    let mut args: Vec<String> = vec!["test".into(), "--doc".into()];

    if target_is_workspace {
        args.push("--workspace".into());
    } else if let Some(project) = project {
        let normalized = normalize_project_name(project);
        args.push("-p".into());
        args.push(normalized);
    } else {
        anyhow::bail!(
            "No project specified. Specify a project name or 'all' to run doc tests for the entire workspace."
        );
    }

    args.push("--all-features".into());

    println!("📚 Running doctests via 'cargo test --doc'...");
    let status = std::process::Command::new("cargo").args(args).status()?;

    if !status.success() {
        anyhow::bail!("Doc tests failed!");
    }

    Ok(())
}
