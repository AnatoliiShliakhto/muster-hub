use mhub_derive::mhub_error;
use std::borrow::Cow;

#[mhub_error]
pub enum DemoError {
    #[error("IO error{}: {source}", format_context(.context))]
    Io {
        #[source]
        source: std::io::Error,
        context: Option<Cow<'static, str>>,
    },

    #[error("Internal error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}

fn main() {}
