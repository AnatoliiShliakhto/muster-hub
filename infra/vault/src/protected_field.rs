//! # Vault Format
//!
//! This module provides `serde` integration for the [`ProtectedPayload`] type.
//! It allows encrypted payloads to be transparently serialized and deserialized
//! as raw byte sequences in data formats like JSON, MessagePack, or database records.
//!
//! ### Usage Example
//!
//! ```rust
//! use mhub_vault::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct UserRecord {
//!     username: String,
//!     /// This field is stored as encrypted bytes but handled as a type-safe payload
//!     #[serde(with = "mhub_vault::vault_format")]
//!     social_security_number: ProtectedPayload<Local>,
//! }
//! ```

use crate::{AeadInPlace, KeyInit, PayloadKind, ProtectedPayload};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

/// Serializes a [`ProtectedPayload`] as its raw internal data.
///
/// This function extracts the internal `Vec<u8>` (which contains the Nonce, 
/// Ciphertext, and Tag) and delegates serialization to it.
///
/// # Errors
/// Returns a serialization error if the underlying [`Serializer`] fails.
pub fn serialize<S, K, A>(
    p: &ProtectedPayload<K, A>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: PayloadKind<A>,
    A: AeadInPlace + KeyInit,
{
    p.data.serialize(s)
}

/// Deserializes a raw byte sequence into a [`ProtectedPayload`].
///
/// This function reconstructs the [`ProtectedPayload`] from the stored data.
/// Note that this does not perform decryption; it only populates the 
/// container for later unsealing via the [`SecurityVault`](crate::SecurityVault).
///
/// # Errors
/// Returns a deserialization error if the input data is not a valid byte sequence.
pub fn deserialize<'de, D, K, A>(
    d: D,
) -> Result<ProtectedPayload<K, A>, D::Error>
where
    D: Deserializer<'de>,
    K: PayloadKind<A>,
    A: AeadInPlace + KeyInit,
{
    let data = Vec::<u8>::deserialize(d)?;
    Ok(ProtectedPayload::from(data))
}
