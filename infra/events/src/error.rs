use std::borrow::Cow;

/// Errors that can occur during event bus operations.
#[mhub_derive::mhub_error]
pub enum EventBusError {
    /// Occurs when an internal dynamic cast fails.
    /// This usually indicates an invariant violation in the type registry.
    #[error("Type mismatch{}: {message}", format_context(.context))]
    TypeMismatch { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// No channel exists for the requested event type.
    #[error("Channel not found{}: {message}", format_context(.context))]
    ChannelNotFound { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Channel exists but with a different kind (broadcast/mpsc/watch).
    #[error("Channel kind mismatch{}: {message}", format_context(.context))]
    ChannelKindMismatch { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// A bounded channel is full and cannot accept more messages.
    #[error("Channel full{}: {message}", format_context(.context))]
    ChannelFull { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Capacity must be greater than zero for bounded channels.
    #[error("Invalid capacity{}: {message}", format_context(.context))]
    InvalidCapacity { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
