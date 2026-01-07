use std::borrow::Cow;

/// Organizations error type.
#[mhub_derive::mhub_error]
pub enum OrganizationError {
    #[error("Internal error{}: {message}", format_context(context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
