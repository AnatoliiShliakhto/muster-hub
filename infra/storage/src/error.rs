use std::borrow::Cow;

/// A specialized [`StorageError`] enum of this crate.
#[mhub_derive::mhub_error]
pub enum StorageError {
    #[error("Directory not found{}: {message}", format_context(.context))]
    DirectoryNotFound { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    #[error("File not found{}: {message}", format_context(.context))]
    FileNotFound { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    #[error("Path traversal security violation{}: {message}", format_context(.context))]
    PathTraversalAttempt { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    #[error("Hardware I/O failure{}: {source}", format_context(.context))]
    Io { source: std::io::Error, context: Option<Cow<'static, str>> },

    #[error("Compression failure{}: {source}", format_context(.context))]
    Compress { source: lz4_flex::block::CompressError, context: Option<Cow<'static, str>> },

    #[error("Decompression failure{}: {source}", format_context(.context))]
    Decompress { source: lz4_flex::block::DecompressError, context: Option<Cow<'static, str>> },
}
