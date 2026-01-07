//! Identity feature slice for JWT + DPoP authentication and Axum middleware

mod error;

use crate::error::{IdentityError, IdentityErrorExt};
use mhub_kernel::domain::registry::InitializedSlice;

/// Identity feature state
#[mhub_derive::mhub_slice]
pub struct Identity {}

/// Initialize the identity feature.
///
/// Extend this function to wire repositories/services when they are implemented.
///
/// # Errors
///
#[cfg(feature = "server")]
pub fn init() -> Result<InitializedSlice, IdentityError> {
    tracing::info!("Identity server slice initialized");

    let inner = IdentityInner {};

    let slice = Identity::new(inner);

    Ok(InitializedSlice::new(slice))
}
