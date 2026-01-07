#[mhub_error::error]
pub enum {{ shortname | pascal_case }}Error {
    /// Error propagated from the underlying logger subsystem.
    #[error(message = "Initialize logger failure", source = mhub_logger::error::LoggerError, kind = "{{ shortname | shouty_snake_case }}_LOGGER_ERROR")]
    Logger,

    /// An unexpected failure.
    #[error(message = "Internal failure", code = 500, kind = "{{ shortname | shouty_snake_case }}_INTERNAL_ERROR")]
    Internal,
}
