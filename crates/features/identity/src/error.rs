use std::borrow::Cow;

/// A specialized [`IdentityError`] enum of this crate.
#[mhub_derive::mhub_error]
pub enum IdentityError {
    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal identity error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
