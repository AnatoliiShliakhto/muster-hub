//! Audit feature slice.
pub mod error;

use crate::error::AuditError;

/// Audit feature inner state.
#[mhub_kernel::macros::mhub_slice]
pub struct Audit {}

/// Initialize the audit feature.
///
/// # Result
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
