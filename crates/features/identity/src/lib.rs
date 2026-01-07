//! IAM (Identity Access Management) feature
mod domain;
mod error;

pub use crate::error::{IdentityError, IdentityErrorExt, Result};
use mhub_kernel::system::registry::InitializedSlice;

/// Identity feature inner state
#[mhub_derive::mhub_slice]
pub struct IdentityInner;

/// Initialize the identity feature
pub fn init() -> Result<InitializedSlice> {
    #[cfg(feature = "server")]
    tracing::info!("Identity & Access Management (IAM) server initialized");

    #[cfg(feature = "client")]
    tracing::info!("Identity & Access Management (IAM) client initialized");

    let slice = Identity::new(IdentityInner);

    Ok(InitializedSlice::new(slice))
}
