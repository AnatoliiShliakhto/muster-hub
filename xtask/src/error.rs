/// High-level errors for the Muster Hub xtask automation suite.
#[mhub_error::error]
pub enum AppError {
    /// Errors related to file system operations (reading manifests, writing artifacts).
    #[error(message = "A filesystem operation failed", source = std::io::Error, kind = "XTASK_IO_ERROR")]
    Io,

    /// I/O formatting failure.
    #[error(message = "Text formatting failure", source = std::fmt::Error, kind = "XTASK_FORMAT_ERROR")]
    Format,

    /// Errors occurring during the generation of code, templates, or assets.
    #[error(
        message = "Failed to generate project assets or source files",
        kind = "XTASK_CODEGEN_FAILURE",
        code = 500
    )]
    Codegen,

    /// Error during parsing.
    #[error(message = "Parsing failure", kind = "XTASK_PARSE_ERROR")]
    Parse,

    /// License-related errors.
    #[error(message = "A license subsystem error", source = mhub_licensing::error::LicenseError, kind = "XTASK_LICENSING_ERROR")]
    License,

    /// Internal error.
    #[error(message = "An unexpected internal error", kind = "XTASK_INTERNAL_ERROR")]
    Internal,
}
