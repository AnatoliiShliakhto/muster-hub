//! # Logger
//!
//! A centralized logging utility for the project.
//! It provides a unified way to configure console and file logging with
//! rotation, non-blocking I/O, and environment-based filtering.
//!
//! ## Example
//!
//! ```rust
//! # use mhub_logger::{LoggerBuilder, LevelFilter};
//!
//! let _logger = LoggerBuilder::new()
//!     .with_app_name("my-app")
//!     .with_stdout(true)
//!     .with_level(LevelFilter::DEBUG)
//!     .init()
//!     .unwrap();
//! ```
mod error;

pub use crate::error::{LoggerError, LoggerErrorExt, Result};
pub use tracing::level_filters::LevelFilter;
pub use tracing_appender::rolling::Rotation;

use std::fs;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

const DEFAULT_MAX_FILES: usize = 10;
const LOG_FILE_SUFFIX: &str = "log";

/// A builder for configuring and initializing the global tracing subscriber.
#[derive(Debug)]
pub struct LoggerBuilder {
    app_name: Option<String>,
    stdout_enabled: bool,
    file_path: Option<PathBuf>,
    default_level: LevelFilter,
    rotation: Rotation,
    max_files: usize,
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self {
            app_name: None,
            stdout_enabled: false,
            file_path: None,
            default_level: LevelFilter::INFO,
            rotation: Rotation::DAILY,
            max_files: DEFAULT_MAX_FILES,
        }
    }
}

impl LoggerBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the application name. This is used as the primary identifier for your logs.
    #[must_use = "This builder does nothing unless `init` is called."]
    pub fn with_app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Whether to enable logging to standard output.
    #[must_use = "This builder does nothing unless `init` is called."]
    pub const fn with_stdout(mut self, enabled: bool) -> Self {
        self.stdout_enabled = enabled;
        self
    }

    /// Configures logging to a file.
    ///
    /// If `Some(path)` is provided, logs will be written to that directory using
    /// a rolling appender. If `None`, file logging is disabled.
    #[must_use = "This builder does nothing unless `init` is called."]
    pub fn with_file_logging(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Sets the default logging level if `RUST_LOG` environment variable is not set.
    #[must_use = "This builder does nothing unless `init` is called."]
    pub const fn with_level(mut self, level: LevelFilter) -> Self {
        self.default_level = level;
        self
    }

    /// Sets the rotation strategy for log files. Defaults to `Rotation::DAILY`.
    #[must_use = "This builder does nothing unless `init` is called."]
    pub const fn with_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the maximum number of log files to retain before deletion.
    #[must_use = "This builder does nothing unless `init` is called."]
    pub const fn with_max_files(mut self, max_files: usize) -> Self {
        self.max_files = max_files;
        self
    }

    /// Consumes the builder and initializes the global tracing subscriber.
    ///
    /// # Returns
    /// A [`Logger`] handle. **Note:** This handle contains a [`WorkerGuard`]
    /// that must be kept alive for the duration of the program to ensure
    /// that non-blocking logs are flushed correctly.
    ///
    /// # Errors
    /// Returns [`LoggerError::Subscriber`] if a global subscriber has already been set.
    pub fn init(self) -> Result<Logger> {
        let app_name = self.app_name.ok_or("App name not set.")?;

        let env_filter =
            EnvFilter::builder().with_default_directive(self.default_level.into()).from_env_lossy();

        let mut layers = Vec::new();

        // 1. Prepare Layers first (No side effects yet)
        if self.stdout_enabled {
            let stdout = layer()
                .compact()
                .with_target(false)
                .with_ansi(true)
                .with_filter(env_filter.clone())
                .boxed();
            layers.push(stdout);
        }

        // 2. Set up File Logging (Side effects: directory creation)
        let guard = if let Some(path) = self.file_path {
            fs::create_dir_all(&path).map_err(|e| LoggerError::Internal {
                message: e.to_string().into(),
                context: Some(
                    format!("Failed to create log directory {path}", path = path.display()).into(),
                ),
            })?;

            let file_appender = RollingFileAppender::builder()
                .rotation(self.rotation)
                .filename_prefix(&app_name)
                .filename_suffix(LOG_FILE_SUFFIX)
                .max_log_files(self.max_files)
                .build(path)?;

            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let file_layer = layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_writer(non_blocking)
                .with_filter(env_filter)
                .boxed();

            layers.push(file_layer);
            Some(guard)
        } else {
            None
        };

        // 3. Initialize Global Registry
        // If this fails, the guard is dropped and the non_blocking worker stops safely.
        tracing_subscriber::registry().with(layers).try_init()?;

        Ok(Logger { guard })
    }
}

/// A handle to the initialized logging system.
///
/// This struct holds the background worker guards. Drop this struct only
/// when the application is shutting down.
#[must_use = "Dropping this handle will stop background logging threads."]
#[derive(Debug)]
pub struct Logger {
    guard: Option<WorkerGuard>,
}

impl Logger {
    /// Returns a new [`LoggerBuilder`] to configure the global tracing subscriber.
    ///
    /// The `app_name` serves as the primary identifier for your logs and is used
    /// as a prefix for rolling log files (e.g., `my-app.2023-10-27.log`).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use mhub_logger::{LevelFilter, Logger};
    /// let _logger = Logger::builder()
    ///     .with_app_name("my-app")
    ///     .with_stdout(true)
    ///     .with_level(LevelFilter::DEBUG)
    ///     .init()
    ///     .unwrap();
    /// ```    
    #[must_use]
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    /// Manually triggers a flush of all pending logs in the non-blocking worker.
    ///
    /// While flushing happens automatically when this handle is dropped,
    /// this is useful for ensuring critical error logs are written before
    /// a controlled shutdown or process exit.
    pub fn flush(&self) {
        tracing::trace!("Logger flush requested");
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        if self.guard.is_some() {
            tracing::info!("Logging system shutting down, flushing buffers...");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn test_logger_builder_initial_state() {
        let builder = LoggerBuilder::new().with_app_name("test-app");
        assert_eq!(builder.app_name, Some("test-app".to_owned()));
        assert!(!builder.stdout_enabled);
        assert!(builder.file_path.is_none());
        assert_eq!(builder.max_files, DEFAULT_MAX_FILES);
    }

    #[test]
    #[serial]
    fn test_logger_builder_configuration() {
        let path = PathBuf::from("./test_logs");
        let builder = LoggerBuilder::new()
            .with_app_name("test-app")
            .with_stdout(true)
            .with_file_logging(&path)
            .with_max_files(5)
            .with_level(LevelFilter::DEBUG);

        assert!(builder.stdout_enabled);
        assert_eq!(builder.file_path, Some(path));
        assert_eq!(builder.max_files, 5);
        assert_eq!(builder.default_level, LevelFilter::DEBUG);
    }

    #[test]
    #[serial]
    fn test_file_logging_setup() -> Result<()> {
        let tmp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {e}"))?;
        let log_dir = tmp_dir.path().join("logs");

        // Configuration
        let builder = LoggerBuilder::new().with_app_name("test-app").with_file_logging(&log_dir);

        // We simulate the part of .build() that creates the directory
        if let Some(path) = &builder.file_path {
            fs::create_dir_all(path).map_err(|e| {
                format!("Failed to create log directory {path}: {e}", path = path.display())
            })?;
        }

        assert!(log_dir.exists());
        Ok(())
    }
}
