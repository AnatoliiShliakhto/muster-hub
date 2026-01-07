pub fn run_all_tests() -> anyhow::Result<()> {
    println!("🧪 Running all workspace tests...");
    let status = std::process::Command::new("cargo")
        .args(["test", "--workspace"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Tests failed!");
    }
    Ok(())
}
