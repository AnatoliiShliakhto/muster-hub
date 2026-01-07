use std::borrow::Cow;

/// A specialized [`FeatureError`] enum of this crate.
#[mhub_derive::mhub_error]
pub enum FeatureError {
    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal feature error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}