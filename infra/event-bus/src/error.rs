/// Errors that can occur during event bus operations.
#[mhub_error::error]
pub enum EventBusError {
    /// Occurs when an internal dynamic cast fails.
    #[error(message = "Event type schema mismatch", code = 500, kind = "BUS_TYPE_MISMATCH")]
    TypeMismatch,

    /// No channel exists for the requested event type.
    #[error(message = "Event channel not found", code = 404, kind = "BUS_CHANNEL_NOT_FOUND")]
    ChannelNotFound,

    /// Channel exists but with a different kind (broadcast/mpsc/watch).
    #[error(message = "Incompatible channel architecture", code = 409, kind = "BUS_KIND_MISMATCH")]
    ChannelKindMismatch,

    /// A bounded channel is full and cannot accept more messages.
    #[error(
        message = "Event bus backpressure limit reached",
        code = 503,
        kind = "BUS_CHANNEL_FULL"
    )]
    ChannelFull,

    /// Capacity must be greater than zero for bounded channels.
    #[error(message = "Invalid bus configuration", code = 400, kind = "BUS_INVALID_CONFIG")]
    InvalidCapacity,
}
