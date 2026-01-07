use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct DockerCompose {
    file_path: &'static str,
}

impl Default for DockerCompose {
    fn default() -> Self {
        Self { file_path: "ops/docker/mhub-dev/docker-compose.yml" }
    }
}

impl DockerCompose {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run a docker-compose command
    fn run(&self, args: &[&str]) -> Result<()> {
        if !Path::new(self.file_path).exists() {
            anyhow::bail!(
                "Docker compose file not found at: {}",
                self.file_path
            );
        }

        let status = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(self.file_path)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| "Failed to execute docker command. Is Docker installed and in your PATH?")?;

        if !status.success() {
            anyhow::bail!("Docker command failed with status: {status}");
        }

        Ok(())
    }

    pub fn up(&self) -> Result<()> {
        println!("🚀 Bringing up infrastructure...");
        self.run(&["up", "-d", "--remove-orphans"])
    }

    pub fn down(&self, volumes: bool) -> Result<()> {
        println!("🛑 Shutting down infrastructure...");
        let mut args = vec!["down"];
        if volumes {
            args.push("-v");
        }
        self.run(&args)
    }

    pub fn logs(&self, service: Option<&str>) -> Result<()> {
        let mut args = vec!["logs", "-f"];
        if let Some(s) = service {
            args.push(s);
        }
        self.run(&args)
    }
}
