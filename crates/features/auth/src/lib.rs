mod domain;
mod error;

pub use crate::error::{Error, Result};

/// Initialize the auth feature
pub fn init() -> Result<()> {
    #[cfg(feature = "api")]
    tracing::info!("Auth API initialized");

    #[cfg(feature = "ui")]
    tracing::info!("Auth UI initialized");

    Ok(())
}