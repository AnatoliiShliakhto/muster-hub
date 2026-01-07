//! Identity feature slice for JWT + `DPop` authentication and Axum middleware

pub mod error;

use crate::error::IdentityError;

/// Identity feature state
#[mhub_kernel::macros::mhub_slice]
pub struct Identity {}

/// Initialize the identity feature.
///
/// Extend this function to wire repositories/services when they are implemented.
///
/// # Errors
///
pub fn init() -> Result<InitializedSlice, IdentityError> {
    let inner = IdentityInner {};

    let slice = Identity::new(inner);

    Ok(InitializedSlice::new(slice))
}
