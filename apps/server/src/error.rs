/// High-level errors for the Muster Hub Server presentation layer.
#[mhub_error::error]
pub enum ServerError {
    /// Failure during the initial bootstrap or configuration phase.
    #[error(message = "Server bootstrap failure", kind = "SERVER_BOOTSTRAP_ERROR")]
    Bootstrap,

    /// SSL/TLS certificate or key configuration is invalid or missing.
    #[error(message = "Invalid ssl configuration", kind = "SERVER_SSL_ERROR")]
    SslConfig,

    /// The server failed to bind to the requested port or address.
    #[error(message = "Server binding failure", kind = "SERVER_BIND_ERROR")]
    Binding,

    /// Failure occurring during the graceful shutdown sequence.
    #[error(message = "Graceful shutdown failure", kind = "SERVER_SHUTDOWN_ERROR")]
    Shutdown,

    /// Error propagated from the underlying database subsystem.
    #[error(message = "Database subsystem failure", source = mhub_database::error::DatabaseError, kind = "SERVER_DATABASE_ERROR")]
    Database,

    /// Error propagated from the underlying logger subsystem.
    #[error(message = "Initialize logger failure", source = mhub_logger::error::LoggerError, kind = "SERVER_LOGGER_ERROR")]
    Logger,

    /// Error propagated from the application configuration loader.
    #[error(message = "Configuration loading failure", source = mhub::kernel::config::ConfigError, kind = "SERVER_CONFIG_ERROR")]
    Config,

    /// Internal error fallback for unhandled server logic.
    #[error(message = "Internal server error", kind = "SERVER_INTERNAL_ERROR")]
    Internal,
}
