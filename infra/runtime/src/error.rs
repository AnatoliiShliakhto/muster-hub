/// Errors that can occur during runtime initialization or operation.
#[mhub_error::error]
pub enum RuntimeError {
    /// Failed to create a runtime due to an I/O error, e.g., failed to create a thread pool
    #[error(
        message = "Failed to initialize runtime",
        source = std::io::Error,
        kind = "RUNTIME_IO_ERROR"
    )]
    Io,
}
