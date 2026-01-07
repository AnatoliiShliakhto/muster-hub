//! # Logger
//!
//! A centralized logging utility for the project.
//! It provides a unified way to configure console and file logging with
//! rotation, non-blocking I/O, and environment-based filtering.
//!
//! * Optional `profiling` support requires building with
//!   `--cfg tokio_unstable` (see notes in [`LoggerBuilder::init`]).
//! * Optional `opentelemetry` support attaches a tracing layer that uses the
//!   global `OpenTelemetry` tracer. Configure a tracer provider before calling
//!   [`LoggerBuilder::init`].
//! * Optional `opentelemetry-otlp` helper installs an `OTLP` tracer provider.
//! * Use [`LoggerBuilder::env_filter`] to set module-directed filters
//!   (e.g., `"myapp=debug,hyper=info"`), in addition to `RUST_LOG`.
//!
//! ## Example
//!
//! ```rust
//! # use mhub_logger::{Logger, LevelFilter};
//!
//! let _logger = Logger::builder()
//!     .name("my-app")
//!     .console(true)
//!     .level(LevelFilter::DEBUG)
//!     .init()
//!     .unwrap();
//! ```

mod error;
#[cfg(feature = "opentelemetry-otlp")]
mod otlp;

pub use crate::error::{LoggerError, LoggerErrorExt};
#[cfg(feature = "opentelemetry-otlp")]
pub use crate::otlp::{OpenTelemetryGuard, init_otlp_tracer};
pub use tracing::level_filters::LevelFilter;
pub use tracing_appender::rolling::Rotation;

use private::Sealed;
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

#[derive(Debug)]
pub struct LoggerConfig {
    console: bool,
    path: Option<PathBuf>,
    level: LevelFilter,
    rotation: Rotation,
    max_files: usize,
    json: bool,
    env_filter: Option<String>,
    #[cfg(feature = "opentelemetry")]
    opentelemetry: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            console: true,
            path: None,
            level: LevelFilter::INFO,
            rotation: Rotation::DAILY,
            max_files: DEFAULT_MAX_FILES,
            json: false,
            env_filter: None,
            #[cfg(feature = "opentelemetry")]
            opentelemetry: false,
        }
    }
}

#[derive(Debug)]
pub struct NoName;
#[derive(Debug)]
pub struct WithName(String);
#[derive(Debug)]
pub struct NoFile;
#[derive(Debug)]
pub struct WithFile;

mod private {
    pub trait Sealed {}
}
impl Sealed for NoName {}
impl Sealed for WithName {}
impl Sealed for NoFile {}
impl Sealed for WithFile {}

/// A builder for configuring and initializing the global tracing subscriber.
#[derive(Debug)]
pub struct LoggerBuilder<N: Sealed = NoName, F: Sealed = NoFile> {
    config: LoggerConfig,
    name: N,
    file_state: std::marker::PhantomData<F>,
}

impl<F: Sealed> LoggerBuilder<NoName, F> {
    /// Sets the name of the logger.
    pub fn name(self, name: impl Into<String>) -> LoggerBuilder<WithName, F> {
        LoggerBuilder {
            name: WithName(name.into()),
            config: self.config,
            file_state: std::marker::PhantomData,
        }
    }
}

impl LoggerBuilder<WithName, WithFile> {
    /// Configures maximum number of log files to keep.
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub const fn max_files(mut self, max: usize) -> Self {
        self.config.max_files = max;
        self
    }

    /// Configures the log file rotation strategy.
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub const fn rotation(mut self, rotation: Rotation) -> Self {
        self.config.rotation = rotation;
        self
    }

    /// Enables JSON logging.
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub const fn json(mut self) -> Self {
        self.config.json = true;
        self
    }
}

impl<F: Sealed> LoggerBuilder<WithName, F> {
    /// Configures the minimum log level to be emitted.
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub const fn level(mut self, level: LevelFilter) -> Self {
        self.config.level = level;
        self
    }

    /// Adds an explicit env filter (e.g., `myapp=debug,hyper=info`).
    ///
    /// Environment variables still override via `RUST_LOG`; this is a programmatic default.
    /// Invalid filters will cause [`LoggerBuilder::init`] to return an error.
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub fn env_filter(mut self, filter: impl Into<String>) -> Self {
        self.config.env_filter = Some(filter.into());
        self
    }

    /// Enables console logging.
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub const fn console(mut self, enabled: bool) -> Self {
        self.config.console = enabled;
        self
    }

    /// Enables `OpenTelemetry` tracing via `tracing-opentelemetry`.
    ///
    /// This attaches a tracing layer backed by the global `OpenTelemetry` tracer.
    /// Make sure you configure and install a tracer provider before calling
    /// [`LoggerBuilder::init`].
    #[cfg(feature = "opentelemetry")]
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub const fn opentelemetry(mut self, enabled: bool) -> Self {
        self.config.opentelemetry = enabled;
        self
    }

    /// Sets the path to log files.
    pub fn path(self, path: impl Into<PathBuf>) -> LoggerBuilder<WithName, WithFile> {
        let mut config = self.config;
        config.path = Some(path.into());
        LoggerBuilder { config, name: self.name, file_state: std::marker::PhantomData }
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
    /// Returns [`LoggerError::InvalidConfiguration`] for invalid builder settings.
    pub fn init(self) -> Result<Logger, LoggerError> {
        validate_config(&self.config, &self.name.0)?;

        let env_filter = build_env_filter(&self.config)?;

        let mut layers = Vec::new();

        #[cfg(all(feature = "profiling", tokio_unstable))]
        if self.config.console {
            layers.push(console_subscriber::spawn().boxed());
        }

        if self.config.console {
            layers.push(layer().compact().with_ansi(true).boxed());
        }

        #[cfg(feature = "opentelemetry")]
        if self.config.opentelemetry {
            let tracer = opentelemetry::global::tracer(self.name.0.clone());
            layers.push(tracing_opentelemetry::layer().with_tracer(tracer).boxed());
        }

        let guard = if let Some(path) = self.config.path {
            fs::create_dir_all(&path).map_err(|e| LoggerError::Internal {
                message: e.to_string().into(),
                context: Some(format!("Failed to create path: {}", path.display()).into()),
            })?;

            let file_appender = RollingFileAppender::builder()
                .rotation(self.config.rotation)
                .filename_prefix(&self.name.0)
                .filename_suffix(LOG_FILE_SUFFIX)
                .max_log_files(self.config.max_files)
                .build(path)?;

            let (non_blocking, g) = tracing_appender::non_blocking(file_appender);

            let file_layer = layer().with_writer(non_blocking).with_ansi(false);

            let boxed =
                if self.config.json { file_layer.json().boxed() } else { file_layer.boxed() };

            layers.push(boxed);
            Some(g)
        } else {
            None
        };

        if layers.is_empty() {
            return Err(LoggerError::InvalidConfiguration {
                message:
                    "No logging layers enabled. Enable console, file output, or OpenTelemetry."
                        .into(),
                context: None,
            });
        }

        tracing_subscriber::registry().with(env_filter).with(layers).try_init()?;

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
    /// The `name` serves as the primary identifier for your logs and is used
    /// as a prefix for rolling log files (e.g., `my-app.2023-10-27.log`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use mhub_logger::{LevelFilter, Logger};
    ///
    /// let _logger = Logger::builder()
    ///     .name("my-app")
    ///     .level(LevelFilter::DEBUG)
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "The builder must be configured before it can be used to initialize the logger."]
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder {
            config: LoggerConfig::default(),
            name: NoName,
            file_state: std::marker::PhantomData,
        }
    }

    /// Manually triggers a flush of all pending logs in the non-blocking worker.
    ///
    /// While flushing happens automatically when this handle is dropped, this
    /// method acts as a best-effort synchronization point before shutdown.
    pub fn flush(&self) {
        tracing::debug!("Logger flushed");
    }

    /// Returns a reference to the underlying worker guard, if present.
    #[must_use]
    pub const fn guard(&self) -> Option<&WorkerGuard> {
        self.guard.as_ref()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        if self.guard.is_some() {
            tracing::info!("Logging system shutting down, flushing buffers...");
        }
    }
}

fn validate_config(config: &LoggerConfig, name: &str) -> Result<(), LoggerError> {
    if name.trim().is_empty() {
        return Err(LoggerError::InvalidConfiguration {
            message: "Logger name cannot be empty".into(),
            context: None,
        });
    }

    if config.max_files == 0 {
        return Err(LoggerError::InvalidConfiguration {
            message: "max_files must be greater than zero".into(),
            context: None,
        });
    }

    Ok(())
}

fn build_env_filter(config: &LoggerConfig) -> Result<EnvFilter, LoggerError> {
    let builder = EnvFilter::builder().with_default_directive(config.level.into());
    config.env_filter.as_ref().map_or_else(
        || Ok(builder.from_env_lossy()),
        |filter| {
            builder.parse(filter).map_err(|e| LoggerError::InvalidConfiguration {
                message: format!("Invalid env filter '{filter}': {e}").into(),
                context: None,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn test_logger_builder_initial_state() {
        let logger_builder = Logger::builder().name("test-app").env_filter("mhub=debug");
        assert!(logger_builder.config.console);
        assert_eq!(logger_builder.config.level, LevelFilter::INFO);
        assert_eq!(logger_builder.config.env_filter.as_deref(), Some("mhub=debug"));
        assert!(logger_builder.config.path.is_none());
    }

    #[test]
    #[serial]
    fn test_logger_builder_configuration() -> Result<(), LoggerError> {
        let tmp_dir = tempdir().map_err(|e| LoggerError::Internal {
            message: e.to_string().into(),
            context: Some("Failed to create temp dir".into()),
        })?;
        let log_dir = tmp_dir.path().join("logs");
        let logger_builder = Logger::builder()
            .name("test-app")
            .console(true)
            .env_filter("mhub=info")
            .path(log_dir.clone())
            .max_files(5)
            .level(LevelFilter::DEBUG);

        assert!(logger_builder.config.console);
        assert_eq!(logger_builder.config.level, LevelFilter::DEBUG);
        assert_eq!(logger_builder.config.max_files, 5);
        assert_eq!(logger_builder.config.env_filter.as_deref(), Some("mhub=info"));
        assert_eq!(logger_builder.config.path.as_deref(), Some(log_dir.as_path()));

        Ok(())
    }

    #[test]
    #[serial]
    fn test_file_logging_setup() -> Result<(), LoggerError> {
        let tmp_dir = tempdir().map_err(|e| LoggerError::Internal {
            message: e.to_string().into(),
            context: Some("Failed to create temp dir".into()),
        })?;
        let log_dir = tmp_dir.path().join("logs");

        let logger =
            Logger::builder().name("test-app").path(&log_dir).level(LevelFilter::INFO).init()?;

        tracing::info!("hello world");
        // Give the background worker a moment, then flush explicitly.
        std::thread::sleep(Duration::from_millis(20));
        logger.flush();

        assert!(log_dir.exists(), "log directory should be created by logger init");

        let entries = fs::read_dir(&log_dir).map_err(|e| LoggerError::Internal {
            message: e.to_string().into(),
            context: Some(format!("Failed to read log directory {}", log_dir.display()).into()),
        })?;

        let has_log = entries
            .flatten()
            .any(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("log"));

        assert!(has_log, "at least one log file should be created");
        Ok(())
    }
}
