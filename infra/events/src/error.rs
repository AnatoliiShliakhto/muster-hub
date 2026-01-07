use std::borrow::Cow;

/// Errors that can occur during event bus operations.
#[mhub_derive::mhub_error]
pub enum EventBusError {
    /// Occurs when an internal dynamic cast fails.
    /// This usually indicates an invariant violation in the type registry.
    #[error("Type mismatch{}: {message}", format_context(.context))]
    TypeMismatch { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}