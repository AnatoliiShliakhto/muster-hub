#[cfg(feature = "client")]
mod client;
mod domain;
mod error;
#[cfg(feature = "server")]
mod server;

pub use crate::error::{FeatureError, FeatureErrorExt};
use mhub_kernel::domain::registry::InitializedSlice;

/// Feature slice state
#[mhub_derive::mhub_slice]
pub struct Feature;

/// Initialize the feature slice
///
/// # Result
///
/// # Errors
///
pub fn init() -> Result<InitializedSlice, FeatureError> {
    #[cfg(feature = "server")]
    tracing::info!("Server feature initialized");

    #[cfg(feature = "client")]
    tracing::info!("Client feature initialized");

    let slice = Feature::new(FeatureInner);

    Ok(InitializedSlice::new(slice))
}
