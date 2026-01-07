//! # Runtime
//!
//! A specialized orchestration layer for the [Tokio](https://tokio.rs) async runtime.
//!
//! This crate provides standardized runtime configurations (profiles) used across
//! the entire workspace to ensure predictable performance and resource usage.
//!
//! ## Profiles
//! * **High Performance**: Optimized for server-side processing with larger stacks and longer keep-alive.
//! * **Memory Efficient**: Optimized for client-side or resource-constrained environments.
//! * **Global**: A shared, lazy-initialized singleton runtime for the entire process.
//!
//! ## Example
//!
//! ```rust,ignore
//! #[mhub_runtime::main(high_performance)]
//! async fn main() -> anyhow::Result<()> {
//!     println!("Running on a high-performance runtime!");
//!     Ok(())
//! }
//! ```

pub use anyhow::Result;
pub use mhub_derive::main;

use anyhow::anyhow;
use std::{sync::OnceLock, thread::available_parallelism, time::Duration};
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, info};

/// The default number of worker threads if detection fails.
const DEFAULT_WORKER_THREADS: usize = 4;
/// The default stack size for threads (3 `MiB`).
const DEFAULT_STACK_SIZE: usize = 3 * 1024 * 1024;
/// Minimum allowed stack size (1 `MiB`).
const MIN_STACK_SIZE: usize = 1024 * 1024;
/// Maximum allowed stack size (16 `MiB`).
const MAX_STACK_SIZE: usize = 16 * 1024 * 1024;
/// How long an idle thread stays alive.
const THREAD_KEEP_ALIVE: Duration = Duration::from_secs(60);

static WORKER_THREADS: OnceLock<usize> = OnceLock::new();

/// Detects the optimal number of worker threads based on environment variables or hardware.
fn get_worker_threads() -> usize {
    *WORKER_THREADS.get_or_init(|| {
        std::env::var("TOKIO_WORKER_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&n| n > 0 && n <= 1024)
            .unwrap_or_else(|| {
                available_parallelism()
                    .map(std::num::NonZero::get)
                    .unwrap_or(DEFAULT_WORKER_THREADS)
            })
    })
}

fn validate_stack_size(stack_size: usize) -> usize {
    stack_size.clamp(MIN_STACK_SIZE, MAX_STACK_SIZE)
}

fn normalize_config(config: &RuntimeConfig) -> RuntimeConfig {
    let thread_name = if config.thread_name.trim().is_empty() {
        "thread-worker".to_owned()
    } else {
        config.thread_name.clone()
    };

    RuntimeConfig {
        worker_threads: config.worker_threads.clamp(1, 1024),
        stack_size: validate_stack_size(config.stack_size),
        thread_name,
        thread_keep_alive: config.thread_keep_alive,
    }
}

/// Configuration for the Tokio runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub worker_threads: usize,
    pub stack_size: usize,
    pub thread_name: String,
    pub thread_keep_alive: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: get_worker_threads(),
            stack_size: DEFAULT_STACK_SIZE,
            thread_name: "thread-worker".to_owned(),
            thread_keep_alive: THREAD_KEEP_ALIVE,
        }
    }
}

impl RuntimeConfig {
    /// Preset for high-throughput server applications.
    #[must_use = "Use this configuration for high-performance server applications"]
    pub fn high_performance() -> Self {
        Self {
            worker_threads: get_worker_threads(),
            stack_size: 4 * 1024 * 1024,
            thread_name: "thread-hp".to_owned(),
            thread_keep_alive: Duration::from_secs(300),
        }
    }

    /// Preset for client applications where memory footprint matters.
    #[must_use = "Use this configuration for low-latency client applications"]
    pub fn memory_efficient() -> Self {
        Self {
            worker_threads: (get_worker_threads() / 2).max(1),
            stack_size: 2 * 1024 * 1024,
            thread_name: "thread-mem".to_owned(),
            thread_keep_alive: Duration::from_secs(30),
        }
    }

    #[must_use = "Customize the number of worker threads for the runtime"]
    pub fn with_worker_threads(mut self, threads: usize) -> Self {
        self.worker_threads = threads.clamp(1, 1024);
        self
    }

    #[must_use = "Customize the stack size for worker threads"]
    pub fn with_stack_size(mut self, size: usize) -> Self {
        self.stack_size = validate_stack_size(size);
        self
    }

    #[must_use = "Customize the thread name"]
    pub fn with_thread_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.thread_name = if name.trim().is_empty() { "thread-worker".to_owned() } else { name };
        self
    }

    #[must_use = "Customize how long idle threads stay alive"]
    pub const fn with_thread_keep_alive(mut self, keep_alive: Duration) -> Self {
        self.thread_keep_alive = keep_alive;
        self
    }
}

/// Creates a new Tokio runtime with a custom stack size.
///
/// This is a convenience function that uses the default [`RuntimeConfig`] but allows
/// you to override the stack size for worker threads. The stack size will be automatically
/// clamped to safe bounds (between 1 `MiB` and 16 `MiB`) to prevent system resource issues.
///
/// The runtime will use the default configuration for:
/// * Worker threads: Auto-detected based on available parallelism
/// * Thread name: "thread-worker"
/// * Keep-alive: 60 seconds for idle threads
///
/// # Parameters
///
/// * `stack_size` - The desired stack size in bytes for each worker thread. Will be
///   automatically clamped to the range `[1_048_576, 16_777_216]` (1-16 `MiB`).
///
/// # Returns
///
/// Returns a [`Result`] containing the initialized [`Runtime`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the Tokio runtime cannot be created, typically due to
/// insufficient system resources or OS-level limitations.
///
/// # Examples
///
/// ```rust,ignore
/// use mhub_runtime::build_runtime;
///
/// // Create a runtime with 4 MiB stack size
/// let runtime = build_runtime(4 * 1024 * 1024)?;
/// runtime.block_on(async {
///     println!("Running with custom stack size");
/// });
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # See Also
///
/// * [`build_service_runtime()`] for the default runtime configuration
/// * [`build_runtime_with_config()`] for full customization control
/// * [`RuntimeConfig::with_stack_size()`] for stack size configuration details
pub fn build_runtime(stack_size: usize) -> Result<Runtime> {
    build_runtime_with_config(&RuntimeConfig::default().with_stack_size(stack_size))
}

/// Creates a new Tokio runtime with a custom configuration.
///
/// This function provides full control over the runtime configuration by accepting
/// a [`RuntimeConfig`] instance. It creates a multithreaded Tokio runtime with all
/// features enabled (I/O, timers, etc.) and applies the specified configuration settings.
///
/// The runtime is built with:
/// * Multi-threaded scheduler for optimal task distribution
/// * All Tokio features enabled (I/O, time, net, signal, etc.)
/// * Configurable worker threads, stack size, thread naming, and keep-alive duration
/// * Debug logging of the configuration for troubleshooting
///
/// # Parameters
///
/// * `config` - A [`RuntimeConfig`] instance that specifies all runtime parameters including
///   worker thread count, stack size, thread naming, and keep-alive duration.
///
/// # Returns
///
/// Returns a [`Result`] containing the initialized [`Runtime`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the Tokio runtime cannot be created. Common causes include:
/// * Insufficient system-resources
/// * Invalid configuration parameters (though most are validated in [`RuntimeConfig`])
/// * OS-level limitations on thread creation
/// * Resource exhaustion
///
/// # Examples
///
/// ```rust,ignore
/// use mhub_runtime::{build_runtime_with_config, RuntimeConfig};
///
/// // Create a runtime with a high-performance preset
/// let config = RuntimeConfig::high_performance();
/// let runtime = build_runtime_with_config(&config)?;
/// runtime.block_on(async {
///     println!("Running with high-performance configuration");
/// });
///
/// // Create a runtime with custom configuration
/// let config = RuntimeConfig::default()
///     .with_worker_threads(8)
///     .with_stack_size(5 * 1024 * 1024)
///     .with_thread_name("custom-worker");
/// let runtime = build_runtime_with_config(&config)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # See Also
///
/// * [`RuntimeConfig`] for available configuration options and presets
/// * [`build_service_runtime()`] for the default configuration
/// * [`build_runtime()`] for quick stack size customization
/// * [`get_global_runtime()`] for accessing the global shared runtime
pub fn build_runtime_with_config(config: &RuntimeConfig) -> Result<Runtime> {
    let config = normalize_config(config);
    debug!(config = ?config, "Building tokio runtime");

    let mut builder = Builder::new_multi_thread();
    builder
        .worker_threads(config.worker_threads)
        .thread_name(&config.thread_name)
        .thread_stack_size(config.stack_size)
        .thread_keep_alive(config.thread_keep_alive);

    builder.enable_all();

    builder.build().map_err(|e| anyhow!("Failed to initialize runtime: {e}"))
}

/// Convenience function to build a runtime using the default configuration.
///
/// This function creates a new Tokio runtime with the default [`RuntimeConfig`] settings,
/// which are suitable for general-purpose service applications. It automatically detects
/// the optimal number of worker threads based on available CPU cores or the
/// `TOKIO_WORKER_THREADS` environment variable.
///
/// The default configuration includes:
/// * Worker threads: Auto-detected based on available parallelism
/// * Stack size: 3 `MiB` per thread
/// * Thread name: "thread-worker"
/// * Keep-alive: 60 seconds for idle threads
///
/// # Returns
///
/// Returns a [`Result`] containing the initialized [`Runtime`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the Tokio runtime cannot be created, typically due to
/// insufficient system resources or OS-level limitations.
///
/// # Examples
///
/// ```rust,ignore
/// use mhub_runtime::build_service_runtime;
///
/// let runtime = build_service_runtime()?;
/// runtime.block_on(async {
///     println!("Running on default service runtime");
/// });
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # See Also
///
/// * [`RuntimeConfig::default()`] for configuration details
/// * [`build_runtime_with_config()`] for custom configurations
/// * [`get_global_runtime()`] for accessing the global shared runtime
pub fn build_service_runtime() -> Result<Runtime> {
    let config = RuntimeConfig::default();
    info!(
        threads = config.worker_threads,
        stack = config.stack_size,
        "Initializing service runtime"
    );
    build_runtime_with_config(&config)
}

static GLOBAL_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Access the lazily initialized global process runtime.
///
/// This is useful for technical components that need access to a runtime but
/// are not called from within an existing async context.
///
/// # Panics
///
/// This function will panic if the Tokio runtime cannot be initialized (e.g.,
/// the OS refuses to allocate threads). This is considered a fatal system error.
pub fn get_global_runtime() -> &'static Runtime {
    GLOBAL_RUNTIME.get_or_init(|| {
        build_service_runtime()
            .expect("CRITICAL: Failed to initialize global infrastructure runtime")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_threads_validation() {
        let config = RuntimeConfig::default().with_worker_threads(0);
        assert_eq!(config.worker_threads, 1);

        let config = RuntimeConfig::default().with_worker_threads(2000);
        assert_eq!(config.worker_threads, 1024);
    }

    #[test]
    fn test_stack_size_validation() {
        let config = RuntimeConfig::default().with_stack_size(100);
        assert_eq!(config.stack_size, MIN_STACK_SIZE);

        let config = RuntimeConfig::default().with_stack_size(100 * 1024 * 1024);
        assert_eq!(config.stack_size, MAX_STACK_SIZE);
    }

    #[test]
    fn test_global_runtime_singleton() {
        let first = get_global_runtime() as *const Runtime;
        let second = get_global_runtime() as *const Runtime;
        assert_eq!(first, second);
    }
}
