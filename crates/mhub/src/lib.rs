pub use mhub_kernel as kernel;
pub use mhub_domain as domain;

#[cfg(feature = "auth")]
pub mod auth {
    pub use mhub_auth as auth;
}

#[cfg(feature = "api")]
pub mod api {
    pub mod router {
        pub use mhub_kernel::api::router::system_router;
    }
}

/// Initialize all enabled features
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "auth")]
    auth::init()?;
    Ok(())
}

/// Feature registry for runtime introspection
pub mod features {
    pub const ENABLED: &[&str] = &[
        #[cfg(feature = "auth")]
        AUTH,
    ];

    #[must_use]
    pub fn is_enabled(name: &str) -> bool {
        ENABLED.contains(&name)
    }
}