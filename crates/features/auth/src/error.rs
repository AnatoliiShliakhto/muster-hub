use std::borrow::Cow;
use thiserror::Error as ThisError;

/// A specialized [`Error`] enum of this crate.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Internal logic errors.
    #[error("{0}")]
    Internal(Cow<'static, str>),
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

/// A specialized [`Result`] type of this crate.
pub type Result<T> = std::result::Result<T, Error>;
