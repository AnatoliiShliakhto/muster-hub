//! A high-performance, domain-isolated cryptographic vault designed for a project ecosystem.
//!
//! This crate provides a unified interface for authenticated encryption with associated data (AEAD),
//! featuring algorithmic agility, memory security, and optimized nonce management.
//!
//! ## Core Architecture
//!
//! The vault is built on three pillars:
//! 1. **Domain Isolation**: Uses type-level markers ([`Local`], [`Fleet`]) to ensure keys from one
//!    security domain cannot be used to unseal data from another.
//! 2. **Algorithmic Agility**: Generic over the `Aead` trait, allowing seamless transitions
//!    between [`Aes`] (hardware-accelerated) and [`ChaCha`] (software-optimized).
//! 3. **Performance**: Utilizes an atomic counter with a random startup prefix for O(1)
//!    nonce generation, suitable for high-frequency operations in async runtimes.
//!
//! ## Security Features
//!
//! - **Zeroization**: Sensitive key material in the [`SecurityVaultBuilder`] is automatically
//!   wiped from memory on a drop or successful build using the `zeroize` crate.
//! - **Context Binding**: Every operation requires an [`AsContext`] (AAD), cryptographically
//!   binding the ciphertext to its specific metadata (like a User ID or Type Name).
//! - **Deterministic Nonces**: Prevents nonce-reuse catastrophes across service restarts by
//!   combining a unique boot-time prefix with an atomic sequence.
//!
//! ## Examples
//!
//! ### Basic Usage via Prelude
//! ```rust
//! use mhub_vault::prelude::*;
//!
//! # fn main() -> VaultResult<()> {
//! let vault = Vault::<Aes>::builder()
//!     .with_derived_keys("master-secret", "salt", "machine-id")
//!     .build()?;
//!
//! // Seal a payload using the vault instance.
//! let secret = "sensitive data".to_string();
//! let sealed = secret.seal_local(&vault)?;
//!
//! // Unseal the sealed payload using the same vault instance.
//! let unsealed = String::unseal(&vault, &sealed)?;
//! assert_eq!(secret, unsealed);
//!
//! # Ok(())
//! # }
//! ```

#![allow(deprecated)]

pub mod error;
pub mod extensions;
pub mod types;
pub mod vault_format;

pub use crate::error::{Error, Result};
use types::{Aes, AsContext, Fleet, Local, PayloadKind, ProtectedPayload};

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{AeadInPlace, Key, KeyInit, Nonce, Tag};
use hkdf::Hkdf;
use rand::{RngCore, thread_rng};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Essential types for quick integration.
pub mod prelude {
    pub use crate::{
        Error as VaultError, Result as VaultResult,
        extensions::VaultExt,
        types::{
            Aes, AsContext, ChaCha, Fleet, Local, PayloadKind,
            ProtectedPayload, Vault,
        },
    };
}

// --- Vault ---

/// High-performance cryptographic vault.
///
/// The vault manages two independent ciphers for different security domains and
/// maintains the state for high-performance nonce generation.
pub struct SecurityVault<A = Aes>
where
    A: AeadInPlace + KeyInit,
{
    pub(crate) local_cipher: A,
    pub(crate) fleet_cipher: A,
    pub(crate) compression: bool,
    nonce_prefix: [u8; 4],
    nonce_counter: AtomicU64,
}

impl<A> SecurityVault<A>
where
    A: AeadInPlace + KeyInit,
{
    /// Returns a new [`SecurityVaultBuilder`] to configure the vault.
    #[must_use]
    pub const fn builder() -> SecurityVaultBuilder<A> {
        SecurityVaultBuilder::new()
    }

    /// Generates a unique, high-performance nonce.
    ///
    /// Combines a 4-byte random boot-prefix with an 8-byte atomic counter.
    #[inline]
    fn next_nonce(&self) -> Nonce<A> {
        let count = self.nonce_counter.fetch_add(1, Ordering::Relaxed);
        let mut b = [0u8; 12];
        b[..4].copy_from_slice(&self.nonce_prefix);
        b[4..].copy_from_slice(&count.to_le_bytes());
        Nonce::<A>::clone_from_slice(&b)
    }

    /// Encrypts and serializes a value into a [`ProtectedPayload`].
    ///
    /// # Errors
    /// Returns [`Error::SerializationFailed`] if JSON encoding fails, or
    /// [`Error::EncryptionFailed`] if the cipher fails.
    pub fn seal_json<K: PayloadKind<A>>(
        &self,
        data: &impl Serialize,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<K, A>> {
        let bytes = serde_json::to_vec(data)?;
        self.seal_raw::<K>(bytes, context)
    }

    /// Unseals a [`ProtectedPayload`] and deserializes it into the target type.
    ///
    /// # Errors
    /// Returns [`Error::DecryptionFailed`] if the context/key is wrong, or
    /// [`Error::SerializationFailed`] if JSON decoding fails.
    pub fn unseal_json<K: PayloadKind<A>, T: DeserializeOwned>(
        &self,
        payload: &ProtectedPayload<K, A>,
        context: &impl AsContext,
    ) -> Result<T> {
        let bytes = self.unseal_raw::<K>(payload, context)?;
        serde_json::from_slice(&bytes).map_err(Error::from)
    }

    /// Encrypts raw bytes into a domain-aware [`ProtectedPayload`].
    ///
    /// This method performs authenticated encryption (AEAD) on the provided `data`.
    /// The `context` is used as Additional Authenticated Data (AAD), meaning it is
    /// not encrypted but is cryptographically bound to the ciphertext.
    ///
    /// # Errors
    /// * [`Error::EncryptionFailed`](Error::EncryptionFailed): If the underlying
    ///   AEAD cipher fails to process the data.
    pub fn seal_raw<K: PayloadKind<A>>(
        &self,
        data: impl AsRef<[u8]>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<K, A>> {
        let cipher = K::select_cipher(self);
        let data =
            self.encrypt_internal(cipher, data.as_ref(), context.as_ctx())?;
        Ok(ProtectedPayload::from(data))
    }

    /// Decrypts a [`ProtectedPayload`] back into raw bytes.
    ///
    /// The unsealing process verifies the authenticity of the data and the
    /// provided `context`. If the context does not exactly match the one used
    /// during sealing, or if the data has been tampered with, this operation will fail.
    ///
    /// # Errors
    /// * [`Error::InvalidPayload`](Error::InvalidPayload): If the payload is
    ///   too short to be valid.
    /// * [`Error::DecryptionFailed`](Error::DecryptionFailed): If the
    ///   cryptographic verification fails (e.g., wrong key, wrong context, or data corruption).
    /// * [`Error::DecompressionFailed`](Error::DecompressionFailed): If the
    ///   payload was compressed and the decompression stream is malformed.
    pub fn unseal_raw<K: PayloadKind<A>>(
        &self,
        payload: &ProtectedPayload<K, A>,
        context: &impl AsContext,
    ) -> Result<Vec<u8>> {
        let cipher = K::select_cipher(self);
        self.decrypt_internal(cipher, payload, context.as_ctx())
    }

    fn encrypt_internal(
        &self,
        cipher: &A,
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let data = if self.compression {
            &lz4_flex::compress_prepend_size(data)
        } else {
            data
        };

        let nonce = self.next_nonce();
        let mut buf = Vec::with_capacity(12 + data.len() + 16);
        buf.extend_from_slice(&nonce);
        buf.extend_from_slice(data);

        let tag = cipher
            .encrypt_in_place_detached(&nonce, aad, &mut buf[12..])
            .map_err(|e| Error::EncryptionFailed(e.to_string()))?;

        buf.extend_from_slice(tag.as_slice());
        Ok(buf)
    }

    fn decrypt_internal(
        &self,
        cipher: &A,
        blob: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        if blob.len() < 28 {
            return Err(Error::InvalidPayload("Short payload"));
        }

        let (nonce_bytes, rest) = blob.split_at(12);
        let (ciphertext, tag_bytes) = rest.split_at(rest.len() - 16);

        let nonce = Nonce::<A>::from_slice(nonce_bytes);
        let tag = Tag::<A>::from_slice(tag_bytes);

        let mut buf = ciphertext.to_vec();
        cipher
            .decrypt_in_place_detached(nonce, aad, &mut buf, tag)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        if self.compression {
            buf = lz4_flex::decompress_size_prepended(&buf)
                .map_err(|e| Error::DecompressionFailed(e.to_string()))?;
        }
        Ok(buf)
    }
}

// --- Builder ---

/// A builder for secure initialization of the [`SecurityVault`].
///
/// Implements `ZeroizeOnDrop` to ensure that raw key material is cleared from
/// memory as soon as the builder is no longer needed.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecurityVaultBuilder<A: AeadInPlace + KeyInit = Aes> {
    #[zeroize(skip)]
    _algo: PhantomData<A>,
    compression: bool,
    local_key: Option<[u8; 32]>,
    fleet_key: Option<[u8; 32]>,
}

impl<A: AeadInPlace + KeyInit> Default for SecurityVaultBuilder<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: AeadInPlace + KeyInit> SecurityVaultBuilder<A> {
    /// Creates a new empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _algo: PhantomData,
            compression: false,
            local_key: None,
            fleet_key: None,
        }
    }

    /// Derives cryptographic keys using HKDF-SHA256.
    ///
    /// # Arguments
    /// * `ikm`: Input Keying Material (Master Password/Secret).
    /// * `salt`: Uniquifies keys across different environments.
    /// * `id`: Binds the [`Local`] key to a specific machine/identity.
    ///
    /// # Panics
    /// *Panics* if HKDF expansion fails, which should never happen with valid inputs.
    #[must_use]
    pub fn with_derived_keys(
        mut self,
        ikm: impl AsRef<[u8]>,
        salt: impl AsRef<[u8]>,
        id: impl AsRef<[u8]>,
    ) -> Self {
        let (_, hk) =
            Hkdf::<Sha256>::extract(Some(salt.as_ref()), ikm.as_ref());
        let mut fleet = [0u8; 32];
        let mut local = [0u8; 32];
        hk.expand(b"v1_fleet", &mut fleet).expect("HKDF");
        let mut info = Vec::from(b"v1_local:");
        info.extend_from_slice(id.as_ref());
        hk.expand(&info, &mut local).expect("HKDF");
        self.fleet_key = Some(fleet);
        self.local_key = Some(local);
        self
    }

    /// Toggles LZ4 compression for all sealed payloads.
    #[must_use]
    pub const fn with_compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }

    /// Finalizes vault construction and `zeroes` the builder.
    ///
    /// # Errors
    /// Returns [`Error::InvalidConfiguration`] if keys were not provided or derived.
    pub fn build(mut self) -> Result<SecurityVault<A>> {
        let l = self
            .local_key
            .ok_or(Error::InvalidConfiguration("No local key"))?;
        let f = self
            .fleet_key
            .ok_or(Error::InvalidConfiguration("No fleet key"))?;
        let mut prefix = [0u8; 4];
        thread_rng().fill_bytes(&mut prefix);

        let v = SecurityVault {
            local_cipher: A::new(Key::<A>::from_slice(&l)),
            fleet_cipher: A::new(Key::<A>::from_slice(&f)),
            compression: self.compression,
            nonce_prefix: prefix,
            nonce_counter: AtomicU64::new(0),
        };
        self.zeroize();
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::SecurityVault;
    use super::prelude::*;

    #[test]
    fn test_vault_builder() {
        let builder = Vault::<ChaCha>::builder()
            .with_derived_keys("master", "salt", "id");

        assert!(builder.local_key.is_some());
        let _ = builder.build().unwrap();
    }

    #[test]
    fn test_nonce_sequence() {
        let vault = Vault::<ChaCha>::builder()
            .with_derived_keys("ikm", "salt", "id")
            .build()
            .unwrap();

        let n1 = vault.next_nonce();
        let n2 = vault.next_nonce();

        assert_ne!(n1, n2);
        let c1 = u64::from_le_bytes(n1[4..].try_into().unwrap());
        let c2 = u64::from_le_bytes(n2[4..].try_into().unwrap());
        assert_eq!(c1 + 1, c2);
    }

    fn setup_vault(compression: bool) -> SecurityVault<ChaCha> {
        Vault::builder()
            .with_derived_keys("ikm", "salt", "id")
            .with_compression(compression)
            .build()
            .expect("Vault should build with derived keys")
    }

    #[test]
    fn test_seal_unseal_raw_local() {
        let vault = setup_vault(false);
        let data = b"sensitive local data";
        let context = b"request-id-456";

        let sealed = vault.seal_raw::<Local>(data, context).unwrap();
        let unsealed = vault.unseal_raw::<Local>(&sealed, context).unwrap();

        assert_eq!(data.as_slice(), unsealed.as_slice());
    }

    #[test]
    fn test_seal_unseal_with_compression() {
        let vault = setup_vault(true);
        let data = b"sensitive local data";
        let context = b"request-id-456";

        let sealed = vault.seal_raw::<Local>(data, context).unwrap();
        let unsealed = vault.unseal_raw::<Local>(&sealed, context).unwrap();

        assert_eq!(data.as_slice(), unsealed.as_slice());
    }

    #[test]
    fn test_seal_unseal_json_fleet() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct MyConfig {
            enabled: bool,
            tokens: Vec<String>,
        }

        let vault = setup_vault(false);

        let original = MyConfig {
            enabled: true,
            tokens: vec!["a".to_owned(), "b".to_owned()],
        };
        let context = b"fleet-context";

        let sealed = vault.seal_json::<Fleet>(&original, context).unwrap();
        let recovered: MyConfig = vault.unseal_json(&sealed, context).unwrap();

        assert_eq!(original, recovered);
    }

    #[test]
    fn test_unseal_fails_with_wrong_context() {
        let vault = setup_vault(false);
        let sealed =
            vault.seal_raw::<Local>(b"data", b"correct-context").unwrap();

        let result = vault.unseal_raw::<Local>(&sealed, b"wrong-context");
        assert!(
            result.is_err(),
            "Decryption should fail if AAD/context mismatch"
        );
    }

    #[test]
    fn test_vault_builder_missing_keys() {
        let result = Vault::<ChaCha>::builder().build();
        assert!(result.is_err(), "Building without keys should fail");
    }
}