//! Audit feature slice.
mod error;

pub use crate::error::{AuditError, AuditErrorExt};
use mhub_kernel::domain::registry::InitializedSlice;

/// Audit feature inner state.
#[mhub_derive::mhub_slice]
pub struct Audit {}

/// Initialize the audit feature.
///
/// # Errors
///
#[cfg(feature = "server")]
pub fn init() -> Result<InitializedSlice, AuditError> {
    tracing::info!("Audit slice initialized");

    let inner = AuditInner {};

    let slice = Audit::new(inner);
    Ok(InitializedSlice::new(slice))
}
