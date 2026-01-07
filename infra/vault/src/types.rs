use crate::engine::Vault;
use aead::{AeadInOut, KeyInit};
use aes_gcm::Aes256Gcm;
use chacha20poly1305::ChaCha20Poly1305;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::Deref;

// --- Aliases ---

pub type Aes = Aes256Gcm;
pub type ChaCha = ChaCha20Poly1305;

pub trait VaultCipher: AeadInOut + KeyInit + 'static {}
impl<T: AeadInOut + KeyInit + 'static> VaultCipher for T {}

// --- Payload format constants ---

/// Payload header version for [`ProtectedPayload`].
pub(crate) const PAYLOAD_VERSION_V1: u8 = 1;

/// Header layout: `[version: u8][flags: u8]`
pub(crate) const HEADER_LEN: usize = 2;

/// AEAD nonce length (96-bit).
pub(crate) const NONCE_LEN: usize = 12;

/// AEAD tag length (128-bit).
pub(crate) const TAG_LEN: usize = 16;

/// Flag bit: payload ciphertext was compressed before encryption.
pub(crate) const FLAG_COMPRESSED: u8 = 1 << 0;

// --- Markers ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Local;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fleet;

// --- Container ---

/// A domain-aware encrypted container for protected data.
///
/// The payload is packed using the following memory layout:
///
/// ```text
/// [V(1)][FLAGS(1)][NONCE(12)][CIPHERTEXT(N)][TAG(16)]
/// ```
///
/// - `V` is the payload format version.
/// - `FLAGS` currently contains the compression bit.
/// - The `Kind` type parameter ensures correct domain usage ([`Local`] or [`Fleet`]).
#[derive(Clone, Serialize, Deserialize)]
pub struct ProtectedPayload<Kind, C = Aes> {
    pub(crate) data: Vec<u8>,
    #[serde(skip)]
    _kind: PhantomData<Kind>,
    #[serde(skip)]
    _cipher: PhantomData<C>,
}

impl<K, C> std::fmt::Debug for ProtectedPayload<K, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtectedPayload").field("data", &self.data).finish()
    }
}

impl<K, C> PartialEq for ProtectedPayload<K, C> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<K, C> Eq for ProtectedPayload<K, C> {}

impl<K, C> std::hash::Hash for ProtectedPayload<K, C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<K, C> ProtectedPayload<K, C> {
    /// Returns the payload format version.
    #[must_use]
    pub fn version(&self) -> Option<u8> {
        self.data.first().copied()
    }

    /// Returns `true` if the payload indicates compression.
    #[must_use]
    pub fn is_compressed(&self) -> bool {
        self.data.get(1).copied().is_some_and(|f| (f & FLAG_COMPRESSED) != 0)
    }

    /// Splits the payload into its constituent cryptographic parts.
    ///
    /// Returns a tuple of `(header, nonce, ciphertext, tag)`.
    #[must_use]
    pub fn split(&self) -> (&[u8], &[u8], &[u8], &[u8]) {
        let (header, rest) = self.data.split_at(HEADER_LEN);
        let (nonce, rest) = rest.split_at(NONCE_LEN);
        let (ciphertext, tag) = rest.split_at(rest.len().saturating_sub(TAG_LEN));
        (header, nonce, ciphertext, tag)
    }
}

impl<K, C> AsRef<[u8]> for ProtectedPayload<K, C> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<K, C> Deref for ProtectedPayload<K, C> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<K, C> From<Vec<u8>> for ProtectedPayload<K, C> {
    fn from(data: Vec<u8>) -> Self {
        Self { data, _kind: PhantomData, _cipher: PhantomData }
    }
}

// --- Dispatch ---

mod private {
    pub trait Sealed {}
    impl Sealed for super::Local {}
    impl Sealed for super::Fleet {}
}

pub trait PayloadKind<C: VaultCipher>: private::Sealed + 'static {
    fn select_cipher(vault: &Vault<C>) -> &C;
}

impl<C: VaultCipher> PayloadKind<C> for Local {
    fn select_cipher(vault: &Vault<C>) -> &C {
        &vault.inner.local_cipher
    }
}

impl<C: VaultCipher> PayloadKind<C> for Fleet {
    fn select_cipher(vault: &Vault<C>) -> &C {
        &vault.inner.fleet_cipher
    }
}

pub trait Tagged {
    const TAG: &'static str;
}

/// Marker trait for types that support vault serialization.
pub trait VaultSerde: Serialize + DeserializeOwned + Tagged {}

impl<K, C> ProtectedPayload<K, C> {
    /// Returns the raw sealed bytes.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}
