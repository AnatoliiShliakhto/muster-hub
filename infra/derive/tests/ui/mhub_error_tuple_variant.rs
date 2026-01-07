use mhub_derive::mhub_error;

#[mhub_error]
pub enum DemoError {
    #[error("IO error: {0}")]
    Io(std::io::Error),
}

fn main() {}
