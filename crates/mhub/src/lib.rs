//! Facade crate for `MusterHub` features and shared modules.
//! Re-exports domain/kernel primitives and aggregates feature initialization.
//! Keep this crate thin: it should compose other crates, not implement business logic.
//!
//! ## Usage
//! - Add `mhub` with the desired feature flags (`server`/`client`).
//! - Call `mhub::init` (server) to register feature slices; extend as new slices appear.

use mhub_database::Database;
pub use mhub_domain as domain;
use mhub_domain::config::ApiConfig;
use mhub_event_bus::EventBus;
pub use mhub_kernel as kernel;
#[cfg(feature = "server")]
pub use mhub_licensing as licensing;

#[cfg(feature = "server")]
pub mod server {
    pub mod router {
        pub use mhub_kernel::server::router::system_router;
    }
}

/// Feature registry for runtime introspection.
pub mod features {
    pub use mhub_audit as audit;
    pub use mhub_identity as identity;
    pub use mhub_organization as organization;

    /// Build-time enabled features (by Cargo feature).
    pub const ENABLED: &[&str] = &[
        #[cfg(feature = "server")]
        "server",
        #[cfg(feature = "client")]
        "client",
        #[cfg(feature = "server")]
        "identity",
        #[cfg(feature = "server")]
        "audit",
        #[cfg(feature = "server")]
        "organization",
        #[cfg(feature = "server")]
        "licensing",
    ];

    #[must_use]
    pub fn is_enabled(name: &str) -> bool {
        ENABLED.contains(&name)
    }
}

/// Initialize all enabled features for server mode.
///
/// # Errors
/// Returns an error if any feature initialization fails.
#[cfg(feature = "server")]
pub fn init(
    config: &ApiConfig,
    database: &Database,
    events: &EventBus,
) -> Result<Vec<domain::registry::InitializedSlice>, Box<dyn std::error::Error>> {
    let mut slices = Vec::new();

    // Audit
    slices.push(features::audit::init()?);

    // Organization
    slices.push(features::organization::init()?);

    // Identity & Access Management (IAM)
    slices.push(features::identity::init()?);

    // Licensing (optional)
    // #[cfg(feature = "mhub-licensing")]
    // {
    //     slices.push(mhub_licensing::init()?);
    // }

    Ok(slices)
}
