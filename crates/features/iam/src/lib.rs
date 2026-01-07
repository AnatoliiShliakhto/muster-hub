//! IAM feature slice.

#[cfg(feature = "client")]
mod client;
mod domain;
mod error;
#[cfg(feature = "server")]
mod server;

pub use crate::error::{IamError, IamErrorExt};
use mhub_kernel::domain::registry::InitializedSlice;

/// Feature inner state
#[mhub_derive::mhub_slice]
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
