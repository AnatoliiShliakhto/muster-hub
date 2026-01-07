//! Kernel utilities shared across slices.
//! Keep this crate lightweight; it re-exports ergonomic helpers for IDs, config loading, and security checks.
//!
//! ## ID generation
//! Use `safe_nanoid!` for URL-safe, unambiguous IDs:
//! ```rust
//! # use mhub_kernel::safe_nanoid;
//! let id = safe_nanoid!();
//! assert_eq!(id.len(), 12);
//! ```
//!
//! ## Config loading (non-wasm)
//! ```rust,ignore
//! #[cfg(not(target_arch = "wasm32"))]
//! # {
//!     use mhub_kernel::config::load_config;
//!     let cfg: serde_json::Value = load_config::<serde_json::Value>(Some("server")).unwrap();
//! # }
//! ```
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
pub mod prelude;
pub mod security;
#[cfg(feature = "server")]
pub mod server;

// Alphabet excludes visually ambiguous characters (I, O, l, 0, 1).
pub const SAFE_ALPHABET: &[char; 55] = &[
    '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L',
    'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f',
    'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub use mhub_domain as domain;
pub use nanoid::nanoid;

/// Generates an unambiguous `NanoID` (no visually confusing characters).
#[macro_export]
macro_rules! safe_nanoid {
    () => {
        // Professional alphabet: 2-9, A-Z (minus I, O), a-z (minus l)
        $crate::nanoid!(12, $crate::SAFE_ALPHABET)
    };
    ($size:expr) => {
        $crate::nanoid!($size, $crate::SAFE_ALPHABET)
    };
}
