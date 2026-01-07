//! # Vault Extensions
//!
//! This module provides the [`VaultExt`] extension trait, which simplifies the process
//! of sealing serializable data into encrypted payloads.
//!
//! By using this trait, you can call cryptographic methods directly on your data structures,
//! reducing boilerplate and ensuring consistent use of type names as cryptographic contexts.

use crate::engine::Vault;
use crate::error::VaultError;
use crate::types::{Fleet, Local, PayloadKind, ProtectedPayload, VaultCipher, VaultSerde};

// --- Extensions ---

/// An extension trait for tagged types to provide ergonomic sealing.
///
/// This trait is automatically implemented for any type that implements [`Serialize`] and [`Tagged`].
pub trait VaultExt: VaultSerde {
    /// Seals the object into a [`Local`] payload.
    ///
    /// The encryption is bound to the local machine instance. The cryptographic
    /// context (AAD) is derived from [`Tagged::TAG`].
    ///
    /// # Results
    /// Returns an encrypted [`ProtectedPayload`] in the local domain.
    ///
    /// # Errors
    /// * [`VaultError::PostcardSerialization`] If the object cannot be serialized.
    /// * [`VaultError::Encryption`] If the AEAD cipher fails.
    fn seal_local<C>(&self, vault: &Vault<C>) -> Result<ProtectedPayload<Local, C>, VaultError>
    where
        C: VaultCipher,
        Self: Sized,
    {
        vault.seal(self)
    }

    /// Seals the object into a [`Fleet`] payload.
    ///
    /// The encryption is bound to the cluster fleet, allowing other nodes with
    /// the same master key to unseal it. The cryptographic context (AAD) is
    /// derived from [`Tagged::TAG`].
    ///
    /// # Results
    /// Returns an encrypted [`ProtectedPayload`] in the fleet domain.
    ///
    /// # Errors
    /// * See [`VaultExt::seal_local`] for failure modes.
    fn seal_fleet<C>(&self, vault: &Vault<C>) -> Result<ProtectedPayload<Fleet, C>, VaultError>
    where
        C: VaultCipher,
        Self: Sized,
    {
        vault.seal::<Fleet, Self>(self)
    }

    /// Unseals a [`ProtectedPayload`] back into the original type.
    ///
    /// This method automatically provides the tagged cryptographic context (AAD).
    ///
    /// # Results
    /// Returns the decoded value.
    ///
    /// # Errors
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::PostcardSerialization`] If the decrypted bytes cannot be parsed.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    fn unseal<K, C>(vault: &Vault<C>, payload: &ProtectedPayload<K, C>) -> Result<Self, VaultError>
    where
        K: PayloadKind<C>,
        C: VaultCipher,
    {
        vault.unseal::<K, Self>(payload)
    }
}

impl<T: VaultSerde> VaultExt for T {}
