//! A high-performance, thread-safe, domain-isolated cryptographic vault.
//!
//! This crate provides a unified interface for authenticated encryption with associated data (AEAD),
//! featuring algorithmic agility, memory security, and robust nonce management.
//!
//! ## Core Architecture
//!
//! The vault is built on three pillars:
//! 1. **Domain Isolation**: Uses type-level markers ([`Local`], [`Fleet`]) to ensure keys from one
//!    security domain cannot be used to unseal data from another.
//! 2. **Algorithmic Agility**: Generic over [`VaultCipher`], allowing seamless transitions
//!    between hardware-accelerated [`Aes`] and software-optimized [`ChaCha`].
//! 3. **Thread-Safe State**: Utilizes an internal `Arc` to share cryptographic state and
//!    configuration efficiently across threads without expensive re-initialization.
//!
//! ## Security Features
//!
//! - **96-bit Random Nonces**: Implements high-entropy random nonce generation for every
//!   operation, ensuring safety against nonce-reuse even across system reboots.
//! - **Context Binding**: Every operation supports Additional Authenticated Data (AAD),
//!   binding ciphertexts to specific metadata or stable identifiers via the [`Tagged`] trait.
//! - **Memory Security**: Sensitive key material in the [`SecurityVaultBuilder`] is
//!   automatically zeroed from memory upon a successful build or drop.
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
//! let unsealed: String = vault.unseal(&sealed)?;
//! // or let unsealed = String::unseal(&vault, &sealed)?;
//! assert_eq!(secret, unsealed);
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### Tagged Payloads
//! ```rust
//! use mhub_vault::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tagged)]
//! #[tagged("v1.user_profile")]
//! struct UserProfile { /* ... */ }
//!
//! # fn main() -> VaultResult<()> {
//! let vault = Vault::<Aes>::builder()
//!     .with_derived_keys("master-secret", "salt", "machine-id")
//!     .build()?;
//!
//! // Seal a payload using the vault instance.
//! let user = UserProfile { };
//! let sealed = user.seal_local_tagged(&vault)?;
//!
//! // Unseal the sealed payload using the same vault instance.
//! let unsealed: UserProfile = vault.unseal_tagged(&sealed)?;
//! assert_eq!(user, unsealed);
//!
//! # Ok(())
//! # }
//! ```

#![allow(deprecated)]

pub mod error;
pub mod extensions;
pub mod protected_field;
pub mod types;

pub use crate::error::{Error, Result};

use aes_gcm::aead::{Key, Nonce, Tag};
use aes_gcm::Aes256Gcm;
use hkdf::Hkdf;
use rand::{thread_rng, RngCore};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::Sha256;
use std::marker::PhantomData;
use std::sync::Arc;
use types::{
    Aes, AsContext, Fleet, Local, PayloadKind, ProtectedPayload, Tagged,
    VaultCipher,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Essential types for quick integration.
pub mod prelude {
    pub use crate::{
        extensions::VaultExt, protected_field,
        types::{
            Aes, AsContext, ChaCha, Fleet, Local, ProtectedPayload, Tagged,
            Vault,
        },
        Error as VaultError,
        Result as VaultResult,
    };
    pub use mhub_derive::Tagged;
}

// --- Vault ---

/// High-performance cryptographic vault.
///
/// The vault manages two independent ciphers for different security domains and
/// maintains the state for high-performance nonce generation.
pub struct InnerSecurityVault<C = Aes>
where
    C: VaultCipher,
{
    pub(crate) local_cipher: C,
    pub(crate) fleet_cipher: C,
    pub(crate) compression: bool,
}

#[derive(Clone)]
pub struct SecurityVault<C = Aes>
where
    C: VaultCipher,
{
    inner: Arc<InnerSecurityVault<C>>,
}

impl<C> SecurityVault<C>
where
    C: VaultCipher,
{
    /// Returns a new [`SecurityVaultBuilder`] to configure the vault.
    #[must_use]
    pub const fn builder() -> SecurityVaultBuilder<C> {
        SecurityVaultBuilder::new()
    }

    /// Generates unique, high-performance nonce.
    #[inline]
    fn next_nonce() -> Nonce<C> {
        let mut nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce);
        Nonce::<C>::clone_from_slice(&nonce)
    }

    /// Encrypts and serializes a value into a [`ProtectedPayload`].
    ///
    /// # Errors
    /// Returns [`Error::SerializationFailed`] if JSON encoding fails, or
    /// [`Error::EncryptionFailed`] if the cipher fails.
    pub fn seal_json<K: PayloadKind<C>>(
        &self,
        data: &impl Serialize,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<K, C>> {
        let bytes = serde_json::to_vec(data)?;
        self.seal_raw::<K>(bytes, context)
    }

    /// Unseals a [`ProtectedPayload`] and deserializes it into the target type.
    ///
    /// # Errors
    /// Returns [`Error::DecryptionFailed`] if the context/key is wrong, or
    /// [`Error::SerializationFailed`] if JSON decoding fails.
    pub fn unseal_json<K: PayloadKind<C>, T: DeserializeOwned>(
        &self,
        payload: &ProtectedPayload<K, C>,
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
    pub fn seal_raw<K: PayloadKind<C>>(
        &self,
        data: impl AsRef<[u8]>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<K, C>> {
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
    pub fn unseal_raw<K: PayloadKind<C>>(
        &self,
        payload: &ProtectedPayload<K, C>,
        context: &impl AsContext,
    ) -> Result<Vec<u8>> {
        let cipher = K::select_cipher(self);
        self.decrypt_internal(cipher, payload, context.as_ctx())
    }

    /// Unseals a [`ProtectedPayload`] back into the original type.
    ///
    /// This method automatically provides the empty cryptographic context (AAD).
    ///
    /// # Errors
    /// * [`Error::DecryptionFailed`](Error::DecryptionFailed): If the context, key, or data is invalid.
    /// * [`Error::SerializationFailed`](Error::SerializationFailed): If the decrypted bytes cannot be parsed into `Self`.
    /// * [`Error::DecompressionFailed`](Error::DecompressionFailed): If the LZ4 stream is corrupt.
    pub fn unseal<K, T>(&self, payload: &ProtectedPayload<K, C>) -> Result<T>
    where
        K: PayloadKind<C>,
        C: VaultCipher,
        T: DeserializeOwned,
    {
        self.unseal_json::<K, T>(payload, &b"")
    }

    /// Unseals a [`ProtectedPayload`] back into the original type.
    ///
    /// This method automatically provides the type's full name as the cryptographic
    /// context (AAD). It will only succeed if the payload was originally sealed
    /// using the type's [`Tagged::TAG`] constant.
    ///
    /// # Errors
    /// * See [`SecurityVault::unseal`] for failure modes.
    pub fn unseal_tagged<K, T>(
        &self,
        payload: &ProtectedPayload<K, C>,
    ) -> Result<T>
    where
        K: PayloadKind<C>,
        T: DeserializeOwned + Tagged,
    {
        self.unseal_json::<K, T>(payload, &T::TAG)
    }

    /// Unseals a [`ProtectedPayload`] back into the original type with a custom context.
    ///
    /// It will only succeed if the payload was originally sealed
    /// using the same type and context.
    ///
    /// # Errors
    /// * See [`SecurityVault::unseal`] for failure modes.
    pub fn unseal_with_ctx<K, T>(
        &self,
        payload: &ProtectedPayload<K, C>,
        context: &impl AsContext,
    ) -> Result<T>
    where
        K: PayloadKind<C>,
        C: VaultCipher,
        T: DeserializeOwned,
    {
        self.unseal_json::<K, T>(payload, context)
    }

    fn encrypt_internal(
        &self,
        cipher: &C,
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let data = if self.inner.compression {
            &lz4_flex::compress_prepend_size(data)
        } else {
            data
        };

        let nonce = Self::next_nonce();
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
        cipher: &C,
        blob: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        if blob.len() < 28 {
            return Err(Error::InvalidPayload("Short payload"));
        }

        let (nonce_bytes, rest) = blob.split_at(12);
        let (ciphertext, tag_bytes) = rest.split_at(rest.len() - 16);

        let nonce = Nonce::<C>::from_slice(nonce_bytes);
        let tag = Tag::<C>::from_slice(tag_bytes);

        let mut buf = ciphertext.to_vec();
        cipher
            .decrypt_in_place_detached(nonce, aad, &mut buf, tag)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        if self.inner.compression {
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
pub struct SecurityVaultBuilder<C: VaultCipher = Aes> {
    #[zeroize(skip)]
    _cipher: PhantomData<C>,
    compression: bool,
    local_key: Option<[u8; 32]>,
    fleet_key: Option<[u8; 32]>,
}

impl<C: VaultCipher> Default for SecurityVaultBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: VaultCipher> SecurityVaultBuilder<C> {
    /// Creates a new empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _cipher: PhantomData,
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
    pub fn build(mut self) -> Result<SecurityVault<C>> {
        let l = self
            .local_key
            .ok_or(Error::InvalidConfiguration("No local key"))?;
        let f = self
            .fleet_key
            .ok_or(Error::InvalidConfiguration("No fleet key"))?;

        let v = InnerSecurityVault {
            local_cipher: C::new(Key::<C>::from_slice(&l)),
            fleet_cipher: C::new(Key::<C>::from_slice(&f)),
            compression: self.compression,
        };
        self.zeroize();
        Ok(SecurityVault { inner: Arc::new(v) })
    }
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use crate::SecurityVault;

    #[test]
    fn test_vault_builder() {
        let builder = Vault::<ChaCha>::builder()
            .with_derived_keys("master", "salt", "id");

        assert!(builder.local_key.is_some());
        let _ = builder.build().unwrap();
    }

    #[test]
    fn test_nonce_sequence() {
        let n1 = SecurityVault::<ChaCha>::next_nonce();
        let n2 = SecurityVault::<ChaCha>::next_nonce();

        assert_ne!(n1, n2);
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
