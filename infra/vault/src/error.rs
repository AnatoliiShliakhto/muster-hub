//! # Vault Errors
//!
//! This module defines the [`Error`] enum and [`Result`] type used throughout 
//! the vault crate for reporting cryptographic, serialization, and configuration failures.

use std::borrow::Cow;
use thiserror::Error as ThisError;

/// A specialized [`Error`] enum for vault-related failures.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Failure during the encryption process.
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Failure during the decryption process.
    ///
    /// This usually indicates an incorrect key, a mismatched cryptographic 
    /// context (AAD), or tampered data.
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Failure during JSON serialization or deserialization.
    #[error("Serialization failed: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    /// Failure during data decompression.
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    /// Failure when the vault or builder is incorrectly configured.
    #[error("Configuration error: {0}")]
    InvalidConfiguration(&'static str),

    /// Failure when the provided payload is malformed or too short.
    #[error("Payload error: {0}")]
    InvalidPayload(&'static str),

    /// Internal fallback for unexpected issues or logic errors.
    #[error("{0}")]
    Internal(Cow<'static, str>)
}

impl From<String> for Error {
    /// Converts a dynamic [`String`] into an [`Error::Internal`] variant.
    fn from(s: String) -> Self {
        Self::Internal(Cow::Owned(s))
    }
}

impl From<&'static str> for Error {
    /// Converts a static string slice into an [`Error::Internal`] variant.
    fn from(s: &'static str) -> Self {
        Self::Internal(Cow::Borrowed(s))
    }
}

/// A specialized [`Result`] type for vault operations.
pub type Result<T> = std::result::Result<T, Error>;