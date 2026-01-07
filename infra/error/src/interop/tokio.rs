use crate::{ErrorCode, ErrorMetadata};
use std::borrow::Cow;

impl ErrorMetadata for tokio::time::error::Elapsed {
    fn code(&self) -> ErrorCode {
        ErrorCode::GatewayTimeout
    }

    fn kind(&self) -> Cow<'static, str> {
        "REQUEST_TIMEOUT".into()
    }

    fn target(&self) -> &'static str {
        "tokio::time"
    }
}
