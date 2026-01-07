pub fn run_all_tests() -> anyhow::Result<()> {
    println!("🧪 Running all workspace tests...");
    let has_nextest = std::process::Command::new("cargo-nextest").arg("--version").output().is_ok();

    let args = if has_nextest {
        vec!["nextest", "run", "--workspace", "--features", "server,client"]
    } else {
        vec!["test", "--workspace", "--features", "server,client"]
    };

    println!("🧪 Running tests via {}...", if has_nextest { "nextest" } else { "cargo test" });
    let status = std::process::Command::new("cargo").args(args).status()?;

    if !status.success() {
        anyhow::bail!("Tests failed!");
    }
    Ok(())
}
