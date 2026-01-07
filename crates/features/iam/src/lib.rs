//! IAM feature slice.

#[cfg(feature = "client")]
mod client;
mod domain;
pub mod error;
#[cfg(feature = "server")]
mod server;

use crate::error::IamError;

/// Feature inner state
#[mhub_kernel::macros::mhub_slice]
pub struct Feature {}

/// Initialize the feature
///
/// # Result
///
/// # Errors
///
pub fn init() -> Result<InitializedSlice, IamError> {
    #[cfg(feature = "server")]
    tracing::info!("IAM server feature initialized");

    let slice = Feature::new(FeatureInner {});

    Ok(InitializedSlice::new(slice))
}
