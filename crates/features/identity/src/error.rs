use std::borrow::Cow;

/// A specialized [`IdentityError`] enum of this crate.
#[mhub_derive::mhub_error]
pub enum IdentityError {
    /// Configuration errors for identity/authentication.
    #[error("Identity config error{}: {message}", format_context(.context))]
    Config { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
    /// Authentication failures.
    #[error("Identity auth error{}: {message}", format_context(.context))]
    Auth { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal identity error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
