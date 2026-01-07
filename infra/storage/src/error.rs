/// A specialized error enum for local and remote filesystem operations.
#[mhub_error::error]
pub enum StorageError {
    /// The requested directory could not be located in the filesystem.
    #[error(message = "Directory not found", code = 404, kind = "STORAGE_DIRECTORY_NOT_FOUND")]
    DirectoryNotFound,

    /// The requested file could not be located on the filesystem.
    #[error(message = "File not found", code = 404, kind = "STORAGE_FILE_NOT_FOUND")]
    FileNotFound,

    /// Access denied due to a detected path traversal attempt or root escape.
    #[error(
        message = "Path traversal security violation",
        code = 403,
        kind = "STORAGE_SECURITY_VIOLATION"
    )]
    PathTraversalAttempt,

    /// Low-level hardware, operating system, or network filesystem I/O failure.
    #[error(
        message = "Filesystem i/o failure",
        source = std::io::Error,
        kind = "STORAGE_IO_ERROR"
    )]
    Io,

    /// Failure during block-level data compression.
    #[error(
        message = "Data compression failure",
        source = lz4_flex::block::CompressError,
        kind = "STORAGE_COMPRESSION_ERROR"
    )]
    Compress,

    /// Failure during data decompression, often indicating data corruption.
    #[error(
        message = "Data decompression failure",
        source = lz4_flex::block::DecompressError,
        kind = "STORAGE_DECOMPRESSION_ERROR"
    )]
    Decompress,
}
