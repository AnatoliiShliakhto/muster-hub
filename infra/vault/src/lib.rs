//! A high-performance, thread-safe, domain-isolated cryptographic vault.
//!
//! This crate provides a unified interface for authenticated encryption with associated data (AEAD),
//! featuring algorithmic agility, memory security, and robust nonce management.
//!
//! ## Payload Format & Versioning
//!
//! Encrypted payloads are stored as a versioned binary blob with an explicit header:
//!
//! ```text
//! [V(1)][FLAGS(1)][NONCE(12)][CIPHERTEXT(N)][TAG(16)]
//! ```
//!
//! The header enables forward-compatible upgrades and ensures that settings such as compression
//! are encoded in the payload itself.
//!
//! ## Nonce Policy
//!
//! This vault uses **random 96-bit nonces** for every encryption operation.
//! This is a standard approach for `AES-GCM` and `ChaCha20Poly1305`, but it is probabilistic.
//! If you expect extremely high-volume encryption per key, consider designing a stricter nonce
//! strategy (e.g., counter-based nonces) and rotating keys appropriately.
//!
//! ## Compression Threat Model
//!
//! Compression (LZ4) is applied **before encryption** when enabled.
//! While correct, it may leak information via ciphertext length in scenarios where an attacker can:
//! 1) influence plaintext, and
//! 2) observe ciphertext sizes.
//!
//! Use compression primarily for internal storage where lengths are not attacker-observable.
//! Disable it for attacker-controlled inputs and public protocols.
//!
//! ## Examples
//!
//! ### Basic Usage via Prelude
//! ```rust
//! use mhub_vault::prelude::*;
//!
//! #[vault_model]
//! struct UserProfile {
//!     id: String,
//!     name: String,
//! }
//!
//! # fn main() -> Result<(), VaultError> {
//!     let vault = Vault::<Aes>::builder()
//!         .derived_keys("master-secret", "salt", "machine-id")?
//!         .build()?;
//!
//! let profile = UserProfile { id: "42".into(), name: "Ada".into() };
//!
//! // Seal to payload, store raw bytes
//! let sealed = profile.seal_local(&vault)?;
//! let bytes: Vec<u8> = sealed.as_slice().to_vec();
//!
//! // Restore from raw bytes
//! let restored: UserProfile = vault.unseal_local(&bytes)?;
//! assert_eq!(profile, restored);
//!
//! # Ok(())
//! # }
//! ```

mod builder;
mod engine;
mod error;
pub mod extensions;
mod types;

pub use builder::VaultBuilder;
pub use engine::Vault;
pub use error::{VaultError, VaultErrorExt};
pub use mhub_derive::vault_model;
pub use serde;
pub use types::{ProtectedPayload, Tagged, VaultSerde};

pub mod prelude {
    pub use crate::engine::Vault;
    pub use crate::error::{VaultError, VaultErrorExt};
    pub use crate::extensions::VaultExt;
    pub use crate::types::{Aes, ChaCha, Fleet, Local, ProtectedPayload, Tagged};
    pub use mhub_derive::vault_model;
}

pub mod algorithms {
    pub use crate::types::{Aes, ChaCha, VaultCipher};
}

pub mod domains {
    pub use crate::types::{Fleet, Local, PayloadKind};
}
