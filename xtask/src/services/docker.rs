use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct DockerCompose {
    file_path: String,
}

impl Default for DockerCompose {
    fn default() -> Self {
        Self { file_path: "ops/docker/mhub-dev/docker-compose.yml".to_owned() }
    }
}

impl DockerCompose {
    /// Creates a Docker Compose helper with the default `compose` file path.
    ///
    /// # Result
    /// Returns a `DockerCompose` configured for `ops/docker/mhub-dev/docker-compose.yml`.
    ///
    /// # Errors
    /// This function does not return errors.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the compose file path (useful for testing/custom setups).
    ///
    /// # Result
    /// Returns a `DockerCompose` configured with the provided file path.
    ///
    /// # Errors
    /// This function does not return errors.
    pub fn with_file_path(path: impl Into<String>) -> Self {
        Self { file_path: path.into() }
    }

    /// Run a docker-compose command
    ///
    /// # Errors
    ///
    pub fn run(&self, args: &[&str]) -> Result<()> {
        if !Path::new(&self.file_path).exists() {
            anyhow::bail!("Docker compose file not found at: {}", self.file_path);
        }

        let status = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.file_path)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(
                || "Failed to execute docker command. Is Docker installed and in your PATH?",
            )?;

        if !status.success() {
            anyhow::bail!("Docker command failed with status: {status}");
        }

        Ok(())
    }

    /// Bring up the infrastructure
    ///
    /// # Result
    /// Returns `Ok(())` after starting the infrastructure.
    ///
    /// # Errors
    /// Returns an error if the Docker Compose command fails.
    pub fn up(&self) -> Result<()> {
        println!("🚀 Bringing up infrastructure...");
        self.run(&["up", "-d", "--remove-orphans"])
    }

    /// Shuts down the infrastructure.
    ///
    /// # Result
    /// Returns `Ok(())` after stopping the infrastructure.
    ///
    /// # Errors
    /// Returns an error if the Docker Compose command fails.
    pub fn down(&self, volumes: bool) -> Result<()> {
        println!("🛑 Shutting down infrastructure...");
        let mut args = vec!["down"];
        if volumes {
            args.push("-v");
        }
        self.run(&args)
    }

    /// Streams logs for a specific service or all services.
    ///
    /// # Result
    /// Returns `Ok(())` after the logs command is started.
    ///
    /// # Errors
    /// Returns an error if the Docker Compose command fails.
    pub fn logs(&self, service: Option<&str>) -> Result<()> {
        let mut args = vec!["logs", "-f"];
        if let Some(s) = service {
            args.push(s);
        }
        self.run(&args)
    }
}

#[test]
fn docker_compose_file_missing_is_reported() {
    let docker = DockerCompose::with_file_path("non-existent.yml");
    let err = docker.run(&["config"]).unwrap_err();
    assert!(err.to_string().contains("not found"), "expected missing file error, got: {err}");
}
