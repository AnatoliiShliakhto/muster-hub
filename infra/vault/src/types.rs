use crate::{Aes256Gcm, Vault};
use aes_gcm::KeyInit;
use aes_gcm::aead::AeadInPlace;
use chacha20poly1305::ChaCha20Poly1305;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::Deref;

// --- Aliases ---

/// Standard AES-256-GCM Algorithm marker.
///
/// This is the default algorithm for the vault, providing high-performance
/// hardware-accelerated encryption on most modern CPUs.
pub type Aes = Aes256Gcm;

/// High-speed ChaCha20-Poly1305 Algorithm marker.
///
/// Optimized for mobile or embedded systems without hardware AES support.
pub type ChaCha = ChaCha20Poly1305;

/// A convenience trait that bundles the requirements for a Vault cipher.
pub trait VaultCipher: AeadInPlace + KeyInit + 'static {}

/// Blanket implementation for any type that meets the requirements.
impl<T: AeadInPlace + KeyInit + 'static> VaultCipher for T {}

// --- Markers ---

/// Marker for data bound to the local machine instance.
///
/// Payloads marked with `Local` are encrypted with a key derived from
/// both the master secret and a unique machine identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Local;

/// Marker for data shared across the cluster fleet.
///
/// Payloads marked with `Fleet` are encrypted with a key derived solely
/// from the master secret, allowing them to be unsealed by any node in the cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fleet;

// --- Container ---

/// A domain-aware encrypted container for protected data.
///
/// This struct implements [`Deref`] for seamless read-only access to the internal
/// byte slice. The data is packed using the following memory layout:
/// `[Nonce (12 bytes)][Ciphertext (N bytes)][Auth Tag (16 bytes)]`.
///
/// The `Kind` type parameter ensures that the payload is treated according to its
/// intended security domain ([`Local`] or [`Fleet`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedPayload<Kind, C = Aes> {
    pub(crate) data: Vec<u8>,
    #[serde(skip)]
    _kind: PhantomData<Kind>,
    #[serde(skip)]
    _cipher: PhantomData<C>,
}

impl<K, C> ProtectedPayload<K, C> {
    /// Splits the payload into its constituent cryptographic parts.
    ///
    /// Returns a tuple of `(nonce, ciphertext, tag)`.
    #[must_use]
    pub fn split(&self) -> (&[u8], &[u8], &[u8]) {
        let (nonce, rest) = self.data.split_at(12);
        let (ciphertext, tag) = rest.split_at(rest.len() - 16);
        (nonce, ciphertext, tag)
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

/// A trait defining the mapping between a payload marker and a vault cipher.
///
/// This trait is **sealed** to ensure that only authorized security domains
/// ([`Local`] and [`Fleet`]) can be used for cryptographic operations.
pub trait PayloadKind<C: VaultCipher>: private::Sealed + 'static {
    /// Selects the appropriate cipher from the vault for this security domain.
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

// --- Context Abstraction ---

/// A trait for types that can be used as a cryptographic context (AAD).
///
/// The context (Additional Authenticated Data) is cryptographically bound to
/// the ciphertext. The data cannot be unsealed unless the exact same context
/// is provided during the decryption process.
pub trait AsContext {
    /// Returns the context as a raw byte slice.
    fn as_ctx(&self) -> &[u8];
}

/// Blanket implementation for anything that can be viewed as a byte slice.
impl<T: AsRef<[u8]>> AsContext for T {
    #[inline]
    fn as_ctx(&self) -> &[u8] {
        self.as_ref()
    }
}

/// A trait for types that provide a stable, unique cryptographic tag.
pub trait Tagged {
    /// A stable string identifier for the type.
    const TAG: &'static str;
}
