//! # Vault Extensions
//!
//! This module provides the [`VaultExt`] extension trait, which simplifies the process
//! of sealing serializable data into encrypted payloads.
//!
//! By using this trait, you can call cryptographic methods directly on your data structures,
//! reducing boilerplate and ensuring consistent use of type names as cryptographic contexts.

use crate::{
    AeadInPlace, Fleet, KeyInit, Local, PayloadKind, ProtectedPayload, Result,
    SecurityVault, types::AsContext,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

// --- Extensions ---

/// An extension trait for [`Serialize`] types to provide ergonomic sealing.
///
/// This trait is automatically implemented for any type that implements [`Serialize`].
/// It binds the encryption to the type's name via [`std::any::type_name`], providing
/// an extra layer of protection against type-confusion attacks.
///
/// ### ⚠️ Warning
/// Since this trait uses the Rust type name as the cryptographic context (AAD), **renaming
/// your struct or moving it to a different module will break the ability to unseal
/// previously encrypted data.**
pub trait VaultExt: Serialize {
    /// Seals the object into a [`Local`] payload.
    ///
    /// The encryption is bound to the local machine instance. The cryptographic
    /// context (AAD) is automatically set to the full path of the type `T`.
    ///
    /// # Errors
    /// * [`Error::SerializationFailed`](crate::Error::SerializationFailed): If the object cannot be serialized to JSON.
    /// * [`Error::EncryptionFailed`](crate::Error::EncryptionFailed): If the AEAD cipher fails.
    fn seal_local<A>(
        &self,
        vault: &SecurityVault<A>,
    ) -> Result<ProtectedPayload<Local, A>>
    where
        A: AeadInPlace + KeyInit;

    /// Seals the object into a [`Local`] payload with a custom cryptographic context.
    ///
    /// The encryption is bound to the local machine instance.
    ///
    /// # Errors
    /// * See [`Self::seal_local`] for failure modes.
    fn seal_local_with_ctx<A>(
        &self,
        vault: &SecurityVault<A>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<Local, A>>
    where
        A: AeadInPlace + KeyInit;

    /// Seals the object into a [`Fleet`] payload.
    ///
    /// The encryption is bound to the cluster fleet, allowing other nodes with
    /// the same master key to unseal it. The cryptographic context (AAD) is
    /// automatically set to the full path of the type `T`.
    ///
    /// # Errors
    /// * See [`Self::seal_local`] for failure modes.
    fn seal_fleet<A>(
        &self,
        vault: &SecurityVault<A>,
    ) -> Result<ProtectedPayload<Fleet, A>>
    where
        A: AeadInPlace + KeyInit;

    /// Seals the object into a [`Fleet`] payload with a custom cryptographic context.
    ///
    /// The encryption is bound to the cluster fleet, allowing other nodes with
    /// the same master key to unseal it.
    ///
    /// # Errors
    /// * See [`Self::seal_local`] for failure modes.
    fn seal_fleet_with_ctx<A>(
        &self,
        vault: &SecurityVault<A>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<Fleet, A>>
    where
        A: AeadInPlace + KeyInit;

    /// Unseals a [`ProtectedPayload`] back into the original type.
    ///
    /// This method automatically provides the type's full name as the cryptographic
    /// context (AAD). It will only succeed if the payload was originally sealed
    /// using the same type and context.
    ///
    /// # Errors
    /// * [`Error::DecryptionFailed`](crate::Error::DecryptionFailed): If the context, key, or data is invalid.
    /// * [`Error::SerializationFailed`](crate::Error::SerializationFailed): If the decrypted bytes cannot be parsed into `Self`.
    /// * [`Error::DecompressionFailed`](crate::Error::DecompressionFailed): If the LZ4 stream is corrupt.
    fn unseal<K, A>(
        vault: &SecurityVault<A>,
        payload: &ProtectedPayload<K, A>,
    ) -> Result<Self>
    where
        K: PayloadKind<A>,
        A: AeadInPlace + KeyInit,
        Self: DeserializeOwned;

    /// Unseals a [`ProtectedPayload`] back into the original type with a custom context.
    ///
    /// It will only succeed if the payload was originally sealed
    /// using the same type and context.
    ///
    /// # Errors
    /// * See [`Self::unseal`] for failure modes.
    fn unseal_with_ctx<K, A>(
        vault: &SecurityVault<A>,
        payload: &ProtectedPayload<K, A>,
        context: &impl AsContext,
    ) -> Result<Self>
    where
        K: PayloadKind<A>,
        A: AeadInPlace + KeyInit,
        Self: DeserializeOwned;
}

impl<T: Serialize> VaultExt for T {
    fn seal_local<A>(
        &self,
        vault: &SecurityVault<A>,
    ) -> Result<ProtectedPayload<Local, A>>
    where
        A: AeadInPlace + KeyInit,
    {
        vault.seal_json::<Local>(self, &std::any::type_name::<T>())
    }

    fn seal_local_with_ctx<A>(
        &self,
        vault: &SecurityVault<A>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<Local, A>>
    where
        A: AeadInPlace + KeyInit,
    {
        vault.seal_json::<Local>(self, context)
    }

    fn seal_fleet<A>(
        &self,
        vault: &SecurityVault<A>,
    ) -> Result<ProtectedPayload<Fleet, A>>
    where
        A: AeadInPlace + KeyInit,
    {
        vault.seal_json::<Fleet>(self, &std::any::type_name::<T>())
    }

    fn seal_fleet_with_ctx<A>(
        &self,
        vault: &SecurityVault<A>,
        context: &impl AsContext,
    ) -> Result<ProtectedPayload<Fleet, A>>
    where
        A: AeadInPlace + KeyInit,
    {
        vault.seal_json::<Fleet>(self, context)
    }

    fn unseal<K, A>(
        vault: &SecurityVault<A>,
        payload: &ProtectedPayload<K, A>,
    ) -> Result<Self>
    where
        K: PayloadKind<A>,
        A: AeadInPlace + KeyInit,
        Self: DeserializeOwned,
    {
        vault.unseal_json::<K, Self>(payload, &std::any::type_name::<T>())
    }

    fn unseal_with_ctx<K, A>(
        vault: &SecurityVault<A>,
        payload: &ProtectedPayload<K, A>,
        context: &impl AsContext,
    ) -> Result<Self>
    where
        K: PayloadKind<A>,
        A: AeadInPlace + KeyInit,
        Self: DeserializeOwned,
    {
        vault.unseal_json::<K, Self>(payload, context)
    }
}
