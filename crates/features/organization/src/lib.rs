//! Organizations feature slice.
mod error;

pub use error::{OrganizationError, OrganizationErrorExt};
use mhub_kernel::domain::registry::InitializedSlice;

/// Organization feature state.
#[mhub_derive::mhub_slice]
pub struct Organization {}

/// Initialize the organization feature.
///
/// # Errors
/// Returns an error if the reparent worker fails to start.
#[cfg(feature = "server")]
pub fn init() -> Result<InitializedSlice, OrganizationError> {
    #[cfg(feature = "server")]
    tracing::info!("Organization slice initialized");

    let inner = OrganizationInner {};

    let slice = Organization::new(inner);
    Ok(InitializedSlice::new(slice))
}

#[cfg(not(feature = "server"))]
pub fn init(_config: &ApiConfig) -> Result<InitializedSlice> {
    #[cfg(feature = "client")]
    tracing::info!("Organization slice initialized (client)");

    let slice = Organization::new(OrganizationInner);
    Ok(InitializedSlice::new(slice))
}
