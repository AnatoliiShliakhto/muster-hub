use std::borrow::Cow;

/// A specialized [`StorageError`] enum of this crate.
#[mhub_derive::mhub_error]
pub enum StorageError {
    #[error("Root directory not configured{}: {message}", format_context(.context))]
    MissingRootDirectory { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    #[error("Directory not found{}: {message}", format_context(.context))]
    DirectoryNotFound { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    #[error("File not found{}: {message}", format_context(.context))]
    FileNotFound { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    #[error("Path traversal attempt detected{}: {message}", format_context(.context))]
    PathTraversalAttempt { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    #[error("I/O error{}: {source}", format_context(.context))]
    Io {
        #[source]
        source: std::io::Error,
        context: Option<Cow<'static, str>>,
    },
    
    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal storage error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}