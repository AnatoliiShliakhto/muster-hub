use serde_json::error::Category;

impl crate::ErrorMetadata for serde_json::Error {
    fn code(&self) -> crate::ErrorCode {
        match self.classify() {
            Category::Data => crate::ErrorCode::UnprocessableEntity,

            Category::Syntax | Category::Eof => crate::ErrorCode::BadRequest,

            Category::Io => crate::ErrorCode::InternalServerError,
        }
    }

    fn kind(&self) -> std::borrow::Cow<'static, str> {
        match self.classify() {
            Category::Syntax => "JSON_SYNTAX_ERROR".into(),
            Category::Data => "JSON_SCHEMA_ERROR".into(),
            Category::Eof => "JSON_EOF".into(),
            Category::Io => "JSON_IO_ERROR".into(),
        }
    }

    fn target(&self) -> &'static str {
        "serde_json"
    }
}
