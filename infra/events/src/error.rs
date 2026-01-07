use thiserror::Error;

/// Errors that can occur during event bus operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Occurs when the internal broadcast channel has been closed.
    #[error("Event bus channel is closed")]
    ChannelClosed,

    /// Occurs when an internal dynamic cast fails.
    /// This usually indicates an invariant violation in the type registry.
    #[error("Type mismatch: expected {0}")]
    TypeMismatch(&'static str),

    /// A generic wrapper for internal system failures.
    #[error("Internal event error: {0}")]
    Internal(&'static str),
}

/// A specialized Result type for event bus operations.
pub type Result<T> = std::result::Result<T, Error>;
