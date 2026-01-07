/// Errors that can occur during logger initialization.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failure when configuring the rolling file appender (e.g., invalid path).
    #[error("Failed to initialize rolling file appender: {0}")]
    Appender(#[from] tracing_appender::rolling::InitError),

    /// Failure when creating the log directory.
    #[error("Failed to create log directory: {0}")]
    Io(#[from] std::io::Error),

    /// Occurs if a global tracing subscriber has already been initialized in the current process.
    #[error("Global dispatcher already initialized: {0}")]
    AlreadyInitialized(#[from] tracing_subscriber::util::TryInitError),
}

/// A specialized Result type for logger operations.
pub type Result<T> = std::result::Result<T, Error>;
