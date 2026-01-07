pub mod error;

use crate::error::{{ shortname | pascal_case }}Error;

/// `{{ shortname | pascal_case }}` slice state
#[mhub_kernel::macros::mhub_slice]
pub struct {{ shortname | pascal_case }};

/// Initialize the `{{ shortname | pascal_case }}` feature slice
///
/// # Result
///
/// # Errors
///
pub fn init() -> Result<InitializedSlice, {{ shortname | pascal_case }}Error> {
    #[cfg(feature = "server")]
    tracing::info!("{{ shortname | pascal_case }} server slice initialized");

    #[cfg(feature = "client")]
    tracing::info!("{{ shortname | pascal_case }} client slice initialized");

    let inner = {{ shortname | pascal_case }}Inner {};

    let slice = {{ shortname | pascal_case }}::new(inner);
    Ok(InitializedSlice::new(slice))
}
