mod error;
mod domain;
mod infra;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "client")]
mod client;

pub use crate::error::{FeatureError, FeatureErrorExt, Result};
use mhub_kernel::system::registry::InitializedSlice;

/// Feature inner state
#[mhub_derive::mhub_slice]
pub struct FeatureInner;

/// Initialize the feature
pub fn init() -> Result<InitializedSlice> {
    #[cfg(feature = "server")]
    tracing::info!("Server feature initialized");

    #[cfg(feature = "client")]
    tracing::info!("Client feature initialized");

    let slice = Feature::new(FeatureInner);

    Ok(InitializedSlice::new(slice))
}
