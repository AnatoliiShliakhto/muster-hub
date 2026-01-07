use std::borrow::Cow;
use thiserror::Error as ThisError;

/// Errors that can occur during logger initialization.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Failure when configuring the rolling file appender (e.g., invalid path).
    #[error("Failed to initialize rolling file appender: {0}")]
    Appender(#[from] tracing_appender::rolling::InitError),

    /// Occurs if a global tracing subscriber has already been initialized in the current process.
    #[error("Global dispatcher already initialized: {0}")]
    AlreadyInitialized(#[from] tracing_subscriber::util::TryInitError),

    /// Internal logic errors.
    #[error("{0}")]
    Internal(Cow<'static, str>),
}

impl From<String> for Error {
    /// Converts a dynamic [`String`] into an [`Error::Internal`] variant.
    fn from(s: String) -> Self {
        Self::Internal(Cow::Owned(s))
    }
}

impl From<&'static str> for Error {
    /// Converts a static string slice into an [`Error::Internal`] variant.
    fn from(s: &'static str) -> Self {
        Self::Internal(Cow::Borrowed(s))
    }
}

/// A specialized Result type for logger operations.
pub type Result<T> = std::result::Result<T, Error>;
