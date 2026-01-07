//! # Vault Extensions
//!
//! This module provides the [`VaultExt`] extension trait, which simplifies the process
//! of sealing serializable data into encrypted payloads.
//!
//! By using this trait, you can call cryptographic methods directly on your data structures,
//! reducing boilerplate and ensuring consistent use of type names as cryptographic contexts.

use crate::types::{AsContext, Tagged, VaultCipher};
use crate::{Fleet, Local, PayloadKind, ProtectedPayload, Result, Vault};
use serde::Serialize;
use serde::de::DeserializeOwned;

// --- Extensions ---

/// An extension trait for [`Serialize`] types to provide ergonomic sealing.
///
/// This trait is automatically implemented for any type that implements [`Serialize`].
pub trait VaultExt: Serialize {
    /// Seals the object into a [`Local`] payload.
    ///
    /// The encryption is bound to the local machine instance. The cryptographic
    /// context (AAD) is empty.
    ///
    /// # Errors
    /// * [`VaultError::SerializationFailed`] If the object cannot be serialized to JSON.
    /// * [`VaultError::EncryptionFailed`] If the AEAD cipher fails.
    fn seal_local<C>(&self, vault: &Vault<C>) -> Result<ProtectedPayload<Local, C>>
    where
        C: VaultCipher,
        Self: Sized,
    {
        vault.seal_json(self, &b"")
    }

    /// Seals the object into a [`Local`] payload.
    ///
    /// The encryption is bound to the local machine instance. The cryptographic
    /// context (AAD) is automatically provided by the type's [`Tagged::TAG`] constant.
    ///
    /// # Errors
    /// * [`VaultError::SerializationFailed`] If the object cannot be serialized to JSON.
    /// * [`VaultError::EncryptionFailed`] If the AEAD cipher fails.
    fn seal_local_tagged<C>(&self, vault: &Vault<C>) -> Result<ProtectedPayload<Local, C>>
    where
        C: VaultCipher,
        Self: Tagged + Sized,
    {
        vault.seal_json(self, &Self::TAG)
    }

    /// Seals the object into a [`Local`] payload with a custom cryptographic context.
    ///
    /// The encryption is bound to the local machine instance.
    ///
    /// # Errors
    /// * See [`VaultExt::seal_local`] for failure modes.
    fn seal_local_with_ctx<C>(
        &self,
        vault: &Vault<C>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<Local, C>>
    where
        C: VaultCipher,
        Self: Sized,
    {
        vault.seal_json::<Local>(self, context)
    }

    /// Seals the object into a [`Fleet`] payload.
    ///
    /// The encryption is bound to the cluster fleet, allowing other nodes with
    /// the same master key to unseal it. The cryptographic context (AAD) is empty.
    ///
    /// # Errors
    /// * See [`VaultExt::seal_local`] for failure modes.
    fn seal_fleet<C>(&self, vault: &Vault<C>) -> Result<ProtectedPayload<Fleet, C>>
    where
        C: VaultCipher,
        Self: Sized,
    {
        vault.seal_json::<Fleet>(self, &b"")
    }

    /// Seals the object into a [`Fleet`] payload.
    ///
    /// The encryption is bound to the cluster fleet, allowing other nodes with
    /// the same master key to unseal it. The cryptographic context (AAD) is
    /// automatically provided by the type's [`Tagged::TAG`] constant.
    ///
    /// # Errors
    /// * See [`VaultExt::seal_local_tagged`] for failure modes.
    fn seal_fleet_tagged<C>(&self, vault: &Vault<C>) -> Result<ProtectedPayload<Fleet, C>>
    where
        C: VaultCipher,
        Self: Tagged + Sized,
    {
        vault.seal_json(self, &Self::TAG)
    }

    /// Seals the object into a [`Fleet`] payload with a custom cryptographic context.
    ///
    /// The encryption is bound to the cluster fleet, allowing other nodes with
    /// the same master key to unseal it.
    ///
    /// # Errors
    /// * See [`VaultExt::seal_local`] for failure modes.
    fn seal_fleet_with_ctx<C>(
        &self,
        vault: &Vault<C>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<Fleet, C>>
    where
        C: VaultCipher,
        Self: Sized,
    {
        vault.seal_json::<Fleet>(self, context)
    }

    /// Unseals a [`ProtectedPayload`] back into the original type.
    ///
    /// This method automatically provides the empty cryptographic context (AAD).
    ///
    /// # Errors
    /// * [`VaultError::DecryptionFailed`] If the context, key, or data is invalid.
    /// * [`VaultError::SerializationFailed`] If the decrypted bytes cannot be parsed into `Self`.
    /// * [`VaultError::DecompressionFailed`] If the LZ4 stream is corrupt.
    fn unseal<K, C>(vault: &Vault<C>, payload: &ProtectedPayload<K, C>) -> Result<Self>
    where
        K: PayloadKind<C>,
        C: VaultCipher,
        Self: DeserializeOwned,
    {
        vault.unseal(payload)
    }

    /// Unseals a [`ProtectedPayload`] back into the original type.
    ///
    /// This method automatically provides the type's full name as the cryptographic
    /// context (AAD).
    ///
    /// # Errors
    /// * See [`VaultExt::unseal`] for failure modes.
    fn unseal_tagged<K, C>(vault: &Vault<C>, payload: &ProtectedPayload<K, C>) -> Result<Self>
    where
        K: PayloadKind<C>,
        C: VaultCipher,
        Self: DeserializeOwned + Tagged,
    {
        vault.unseal_tagged(payload)
    }

    /// Unseals a [`ProtectedPayload`] back into the original type with a custom context.
    ///
    /// It will only succeed if the payload was originally sealed
    /// using the same type and context.
    ///
    /// # Errors
    /// * See [`VaultExt::unseal`] for failure modes.
    fn unseal_with_ctx<K, C>(
        vault: &Vault<C>,
        payload: &ProtectedPayload<K, C>,
        context: &impl AsContext,
    ) -> Result<Self>
    where
        K: PayloadKind<C>,
        C: VaultCipher,
        Self: DeserializeOwned,
    {
        vault.unseal_with_ctx(payload, context)
    }
}

impl<T: Serialize> VaultExt for T {}
