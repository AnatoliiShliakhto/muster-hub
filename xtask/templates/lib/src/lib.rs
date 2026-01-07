mod error;

pub use crate::error::{LibError, LibErrorExt, Result};

pub fn init() -> Result<()> {
    Ok(())
}
