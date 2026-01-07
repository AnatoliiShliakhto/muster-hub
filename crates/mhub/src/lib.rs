use mhub_database::Database;
pub use mhub_domain as domain;
use mhub_domain::config::ApiConfig;
pub use mhub_kernel as kernel;
use mhub_kernel::system::registry::InitializedSlice;
#[cfg(feature = "server")]
pub use mhub_licensing as licensing;

#[cfg(feature = "server")]
pub mod server {
    pub mod router {
        pub use mhub_kernel::server::router::system_router;
    }
}

/// Feature registry for runtime introspection
pub mod features {
    pub const ENABLED: &[&str] = &[];

    #[must_use]
    pub fn is_enabled(name: &str) -> bool {
        ENABLED.contains(&name)
    }

    pub use mhub_identity as identity;
}

/// Initialize all enabled features
// #[cfg(all(feature = "server", not(feature = "client")))]
#[cfg(feature = "server")]
pub fn init(
    _config: &ApiConfig,
    _database: &Database,
) -> Result<Vec<InitializedSlice>, Box<dyn std::error::Error>> {
    let mut slices = vec![];

    // Identity & Access Management (IAM)
    slices.push(features::identity::init()?);

    Ok(slices)
}