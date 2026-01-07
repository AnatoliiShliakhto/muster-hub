use crate::{ErrorCode, ErrorMetadata};
use std::borrow::Cow;
use std::io::{Error, ErrorKind};

impl ErrorMetadata for Error {
    fn code(&self) -> ErrorCode {
        match self.kind() {
            ErrorKind::InvalidInput => ErrorCode::BadRequest,
            ErrorKind::PermissionDenied => ErrorCode::Forbidden,
            ErrorKind::NotFound => ErrorCode::NotFound,
            ErrorKind::TimedOut => ErrorCode::RequestTimeout,
            ErrorKind::AlreadyExists => ErrorCode::Conflict,
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected => ErrorCode::BadGateway,
            _ => ErrorCode::InternalServerError,
        }
    }

    fn kind(&self) -> Cow<'static, str> {
        match self.kind() {
            ErrorKind::NotFound => "IO_NOT_FOUND".into(),
            ErrorKind::PermissionDenied => "IO_ACCESS_DENIED".into(),
            ErrorKind::TimedOut => "IO_TIMEOUT".into(),
            ErrorKind::AlreadyExists => "IO_ALREADY_EXISTS".into(),
            _ => "IO_ERROR".into(),
        }
    }

    fn target(&self) -> &'static str {
        "std::io"
    }
}
