use anyhow::{Context, Result};
use mhub_licensing::generator::{generate_keypair, generate_secret};
use std::fs;
use std::process::Command;
use crate::models::keyset::Keyset;

/// Tools required for development
const REQUIRED_TOOLS: &[(&str, &str)] = &[
    ("cargo-generate", "cargo-generate"),
    ("dx", "dioxus-cli"),
    ("cargo-audit", "cargo-audit"),
    ("nextest", "cargo-nextest"),
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
            println!("✅ {bin} is already installed. Trying update...");
        } else {
            println!("📥 Installing {package}...");
        }
        run_command("cargo", &["install", package, "--locked"])?;
    }

    let installed_targets_output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .with_context(|| "Failed to list installed rustup targets")?;

    let installed_targets = String::from_utf8_lossy(&installed_targets_output.stdout);

    for target in REQUIRED_TARGETS {
        if installed_targets.contains(target) {
            println!("✅ Target {target} is already installed.");
            continue;
        }

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
    if fs::metadata("private/keyset").is_ok() {
        return Ok(());
    }

    let (master_key, public_key) = generate_keypair()?;
    let salt = generate_secret()?;

    let keyset = Keyset {
        master_key: master_key.to_bytes(),
        public_key: public_key.to_bytes(),
        salt,
    };
    let keyset_bytes = postcard::to_stdvec(&keyset)
        .context("Binary serialization failed")?;

    fs::create_dir_all("private")?;
    fs::write("private/keyset", keyset_bytes)?;

    println!("🔑 Generated keyset successfully: 'private/keyset'.");
    println!("⚠️ Attention! This keyset is sensitive and should be kept private.");
    println!("You can use it to sign licenses for development purposes.");
    Ok(())
}

fn warn_missing_node() {
    println!("\n⚠️  Node.js not found!");
    println!("Node.js is required for asset management and frontend builds.");
    println!("Please download it from: https://nodejs.org/");
    println!("After installing, restart your terminal and run 'cargo xtask setup' again.\n");
}
