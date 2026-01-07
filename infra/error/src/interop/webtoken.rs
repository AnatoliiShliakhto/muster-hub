use crate::{ErrorCode, ErrorMetadata};
use std::borrow::Cow;

impl ErrorMetadata for jsonwebtoken::errors::Error {
    fn code(&self) -> ErrorCode {
        use jsonwebtoken::errors::ErrorKind;
        match self.kind() {
            ErrorKind::ExpiredSignature => ErrorCode::Unauthorized,
            ErrorKind::InvalidToken | ErrorKind::InvalidSignature => ErrorCode::Forbidden,
            _ => ErrorCode::InternalServerError,
        }
    }

    fn kind(&self) -> Cow<'static, str> {
        format!("JWT_{:?}", self.kind()).to_uppercase().into()
    }

    fn target(&self) -> &'static str {
        "jsonwebtoken"
    }
}
