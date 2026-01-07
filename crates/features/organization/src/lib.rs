//! Organizations feature slice.
pub mod error;

use error::OrganizationError;

/// Organization feature state.
#[mhub_kernel::macros::mhub_slice]
pub struct Organization {}

/// Initialize the organization feature.
///
/// # Errors
///
pub fn init() -> Result<InitializedSlice, OrganizationError> {
    #[cfg(feature = "server")]
    tracing::info!("Organization server slice initialized");

    #[cfg(feature = "client")]
    tracing::info!("Organization client slice initialized");

    let inner = OrganizationInner {};

    let slice = Organization::new(inner);
    Ok(InitializedSlice::new(slice))
}
