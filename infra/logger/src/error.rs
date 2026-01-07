use std::borrow::Cow;

/// Errors that can occur during logger initialization.
#[mhub_derive::mhub_error]
pub enum LoggerError {
    /// Failure when configuring the rolling file appender (e.g., invalid path).
    #[error("Rolling file appender error{}: {source}", format_context(.context))]
    Appender {
        #[source]
        source: tracing_appender::rolling::InitError,
        context: Option<Cow<'static, str>>,
    },

    /// Occurs if a global tracing subscriber has already been initialized in the current process.
    #[error("Tracing subscriber error{}: {source}", format_context(.context))]
    Subscriber {
        #[source]
        source: tracing_subscriber::util::TryInitError,
        context: Option<Cow<'static, str>>,
    },

    /// Internal logic errors.
    #[error("Internal logger error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}