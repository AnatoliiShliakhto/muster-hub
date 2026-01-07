use thiserror::Error as ThisError;

/// Errors that can occur during event bus operations.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Occurs when an internal dynamic cast fails.
    /// This usually indicates an invariant violation in the type registry.
    #[error("Type mismatch: expected {0}")]
    TypeMismatch(&'static str),
}

/// A specialized [`Result`] type for event bus operations.
pub type Result<T> = std::result::Result<T, Error>;
