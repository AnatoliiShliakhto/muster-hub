//pub use mhub_core::prelude::*;

// 2. Namespace business modules
#[cfg(feature = "identity")]
pub mod identity {
    pub use mhub_identity::*;
}

// 3. Namespace technical modules
#[cfg(feature = "api")]
pub mod api {
    pub use mhub_core::api::*;
}