use mhub_derive::mhub_error;

#[mhub_error]
pub enum DemoError {
    #[error("IO error: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

fn main() {}
