use crate::{ErrorCode, ErrorMetadata};
use std::borrow::Cow;

impl ErrorMetadata for surrealdb::Error {
    fn code(&self) -> ErrorCode {
        let msg = self.to_string();
        if msg.contains("exist") && msg.contains("already") {
            ErrorCode::Conflict
        } else if msg.contains("permissions") || msg.contains("authorized") {
            ErrorCode::Forbidden
        } else if msg.contains("authentication") {
            ErrorCode::Unauthorized
        } else {
            ErrorCode::InternalServerError
        }
    }

    fn kind(&self) -> Cow<'static, str> {
        "SURREAL_DATABASE".into()
    }
    fn target(&self) -> &'static str {
        "surrealdb"
    }
}
