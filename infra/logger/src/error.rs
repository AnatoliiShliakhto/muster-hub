/// Errors that can occur during logger initialization or telemetry operation.
#[mhub_error::error]
pub enum LoggerError {
    /// Failure during local file operations such as directory creation or log rotation.
    #[error(
        message = "Log filesystem i/o failure",
        source = std::io::Error,
        kind = "LOGGER_IO_ERROR"
    )]
    Io,

    /// Provided configuration parameters are invalid or incompatible.
    #[error(message = "Invalid logger configuration", code = 400, kind = "LOGGER_CONFIG_ERROR")]
    InvalidConfiguration,

    /// Failure specific to the rolling file appender setup (e.g., locked files).
    #[error(message = "Log file appender initialization failed", kind = "LOGGER_APPENDER_ERROR")]
    Appender,

    /// Indicates that a global tracing subscriber is already active in the process.
    #[error(
        message = "Tracing subscriber conflict",
        source = tracing_subscriber::util::TryInitError,
        kind = "LOGGER_SUBSCRIBER_ERROR"
    )]
    Subscriber,

    /// Failure while connecting to or configuring the `OpenTelemetry` collector.
    #[cfg(feature = "opentelemetry-otlp")]
    #[error(
        message = "Telemetry exporter connection failure",
        source = opentelemetry_otlp::ExporterBuildError,
        code = 502,
        kind = "LOGGER_TELEMETRY_ERROR"
    )]
    OpenTelemetry,
}
