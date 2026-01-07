use crate::services::licensing::{generate_keypair, generate_secret};
use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

/// Tools required for development
const REQUIRED_TOOLS: &[(&str, &str)] = &[
    ("cargo-generate", "cargo-generate"),
    ("dx", "dioxus-cli"),
    ("cargo-audit", "cargo-audit"),
];

/// Targets required for cross-platform/WASM development
const REQUIRED_TARGETS: &[&str] = &["wasm32-unknown-unknown"];

pub fn setup_project() -> Result<()> {
    install_dependencies()?;
    generate_keyset()?;
    Ok(())
}

fn install_dependencies() -> Result<()> {
    println!("🛠️  Starting MusterHub development setup...");

    check_node_environment()?;

    for (bin, package) in REQUIRED_TOOLS {
        if is_tool_installed(bin) {
            println!("✅ {bin} is already installed.");
        } else {
            println!("📥 Installing {package}...");
            run_command("cargo", &["install", package])?;
        }
    }

    for target in REQUIRED_TARGETS {
        println!("🦀 Adding rustup target: {target}...");
        run_command("rustup", &["target", "add", target])?;
    }

    println!("\n✨ Setup complete! You are ready to develop for MusterHub.");
    Ok(())
}

fn is_tool_installed(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute {cmd}"))?;

    if !status.success() {
        anyhow::bail!("Command '{cmd} {args:?}' failed with status {status}");
    }
    Ok(())
}

fn check_node_environment() -> Result<()> {
    if is_tool_installed("node") {
        println!("✅ Node.js is installed.");

        // If the node.js is there, ensure ncu (npm-check-updates) is installed globally
        if is_tool_installed("ncu") {
            println!("✅ ncu (npm-check-updates) is installed.");
        } else {
            println!("📥 Installing ncu globally via npm...");
            // Use 'cmd' on Windows or 'sh' on Unix for npm as it's often a script
            let npm_cmd = if cfg!(windows) { "npm.cmd" } else { "npm" };
            run_command(npm_cmd, &["install", "-g", "npm-check-updates"])?;
        }
    } else {
        warn_missing_node();
    }
    Ok(())
}

fn generate_keyset() -> Result<()> {
    let (master_key, public_key) = generate_keypair()?;
    let salt = generate_secret()?;

    fs::create_dir_all("private/licenses").ok();
    fs::write("private/master-key", master_key.to_bytes())?;
    fs::write("private/public-key", public_key.to_bytes())?;
    fs::write("private/salt", salt)?;

    println!("🔑 Generated keyset successfully.");
    Ok(())
}

fn warn_missing_node() {
    println!("\n⚠️  Node.js not found!");
    println!("Node.js is required for asset management and frontend builds.");
    println!("Please download it from: https://nodejs.org/");
    println!(
        "After installing, restart your terminal and run 'cargo xtask setup' again.\n"
    );
}
