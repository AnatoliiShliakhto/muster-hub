//! # Vault Errors
//!
//! This module defines the [`VaultError`] enum and [`Result`] type used throughout
//! the vault crate for reporting cryptographic, serialization, and configuration failures.

use std::borrow::Cow;

/// A specialized [`VaultError`] enum for vault-related failures.
#[mhub_derive::mhub_error]
pub enum VaultError {
    /// Failure during the encryption process.
    #[error("Encryption error{}: {message}", format_context(.context))]
    Encryption { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Failure during the decryption process.
    ///
    /// This usually indicates an incorrect key, a mismatched cryptographic
    /// context (AAD), or tampered data.
    #[error("Decryption error{}: {message}", format_context(.context))]
    Decryption { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Failure during Postcard serialization or deserialization.
    #[error("Serialization error{}: {source}", format_context(.context))]
    Serialization { source: postcard::Error, context: Option<Cow<'static, str>> },

    /// Failure during data decompression.
    #[error("Decompression error{}: {message}", format_context(.context))]
    Decompression { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Failure when the vault or builder is incorrectly configured.
    #[error("Invalid configuration{}: {message}", format_context(.context))]
    InvalidConfiguration { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Failure when the provided payload is malformed or too short.
    #[error("Invalid payload{}: {message}", format_context(.context))]
    InvalidPayload { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal vault error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
