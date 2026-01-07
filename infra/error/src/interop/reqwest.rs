use crate::{ErrorCode, ErrorMetadata};
use std::borrow::Cow;

impl ErrorMetadata for reqwest::Error {
    fn code(&self) -> ErrorCode {
        if self.is_timeout() {
            ErrorCode::GatewayTimeout
        } else if self.is_connect() {
            ErrorCode::BadGateway
        } else if let Some(status) = self.status() {
            if status.is_client_error() {
                ErrorCode::InternalServerError
            } else {
                ErrorCode::BadGateway
            }
        } else if self.is_decode() {
            ErrorCode::BadGateway
        } else {
            ErrorCode::InternalServerError
        }
    }

    fn kind(&self) -> Cow<'static, str> {
        if self.is_timeout() {
            "HTTP_TIMEOUT".into()
        } else if self.is_connect() {
            "HTTP_CONNECT_ERROR".into()
        } else if self.is_decode() {
            "HTTP_DECODE_ERROR".into()
        } else if self.is_status() {
            "HTTP_UPSTREAM_ERROR".into()
        } else {
            "HTTP_CLIENT_ERROR".into()
        }
    }

    fn target(&self) -> &'static str {
        "reqwest"
    }
}
