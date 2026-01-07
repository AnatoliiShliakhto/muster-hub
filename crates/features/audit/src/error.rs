use std::borrow::Cow;

/// Audit slice error type.
#[mhub_derive::mhub_error]
pub enum AuditError {
    #[error("Audit error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
