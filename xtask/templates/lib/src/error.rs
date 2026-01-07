use std::borrow::Cow;

/// A specialized [`LibError`] enum of this crate.
#[mhub_derive::mhub_error]
pub enum LibError {
    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal library error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}