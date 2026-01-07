use aead::Nonce;
use aead::inout::InOutBuf;
use getrandom::fill;
use std::sync::Arc;

use crate::builder::VaultBuilder;
use crate::domains::{Fleet, Local};
use crate::error::{VaultError, VaultErrorExt};
use crate::types::{
    Aes, FLAG_COMPRESSED, HEADER_LEN, NONCE_LEN, PAYLOAD_VERSION_V1, PayloadKind, ProtectedPayload,
    TAG_LEN, VaultCipher, VaultSerde,
};

/// High-performance cryptographic vault.
///
/// The vault manages two independent ciphers for different security domains and
/// maintains the state for high-performance nonce generation.
#[allow(unreachable_pub)]
#[derive(Debug)]
pub struct VaultInner<C = Aes>
where
    C: VaultCipher,
{
    pub local_cipher: C,
    pub fleet_cipher: C,
    pub compression: bool,
}

/// A thread-safe, high-performance container for cryptographic operations.
///
/// `Vault` serves as the primary interface for encrypting and decrypting data within
/// the application. It wraps an inner vault state in an [`Arc`], making it cheaply clonable
/// and safe to share across threads or asynchronous tasks.
///
/// ### Ciphers & Domains
/// The vault manages two independent cryptographic domains:
/// * **Local Domain**: Used for data sensitive to the local node.
/// * **Fleet Domain**: Used for data shared across the entire cluster/fleet.
///
/// ### Generic Parameters
/// * `C`: The cipher implementation. Defaults to [`Aes`] (AES-256-GCM) for high performance
///   and hardware acceleration support.
///
/// ### Example
/// ```rust
/// use mhub_vault::prelude::*;
///
/// // Create a default AES-based vault
/// # fn main() -> Result<(), VaultError> {
/// let vault = Vault::<Aes>::builder()
///     .derived_keys("ikm", "salt", "my-machine-id")?
///     .build()?;
///
/// #[vault_model(tag = "v1.user_profile")]
/// struct UserProfile {
///     id: String,
///     name: String,
/// }
///
/// let profile = UserProfile { id: "42".into(), name: "Ada".into() };
///
/// // Seal to payload, store raw bytes
/// let sealed = profile.seal_local(&vault)?;
/// let bytes: Vec<u8> = sealed.as_slice().to_vec();
///
/// // Restore from raw bytes
/// let restored: UserProfile = vault.unseal_local(&bytes)?;
/// assert_eq!(profile, restored);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Vault<C = Aes>
where
    C: VaultCipher,
{
    pub(crate) inner: Arc<VaultInner<C>>,
}

impl<C: VaultCipher> Clone for Vault<C> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<C> Vault<C>
where
    C: VaultCipher,
{
    /// Returns a new [`VaultBuilder`] to configure the vault.
    ///
    /// # Results
    /// Returns a new builder instance.
    ///
    /// # Errors
    /// None.
    #[must_use]
    pub fn builder() -> VaultBuilder<C> {
        VaultBuilder::<C>::new()
    }

    /// Generates unique, high-performance nonce.
    #[inline]
    fn next_nonce() -> Nonce<C> {
        let mut nonce = Nonce::<C>::default();
        fill(&mut nonce).expect("System RNG unavailable for nonce generation");
        nonce
    }

    /// Seals a value using `postcard` (compact binary format).
    ///
    /// The cryptographic context is taken from [`Tagged::TAG`].
    ///
    /// # Results
    /// Returns an encrypted [`ProtectedPayload`] bound to the type tag.
    ///
    /// # Errors
    /// * [`VaultError::PostcardSerialization`] If the value cannot be serialized.
    /// * [`VaultError::Encryption`] If the AEAD encryption fails.
    pub fn seal<K, T>(&self, data: &T) -> Result<ProtectedPayload<K, C>, VaultError>
    where
        K: PayloadKind<C>,
        T: VaultSerde,
    {
        let bytes = postcard::to_stdvec(data).context("Postcard encoding failed")?;
        self.seal_bytes::<K>(bytes.as_slice(), T::TAG.as_bytes())
    }

    /// Encrypts raw bytes into a domain-aware [`ProtectedPayload`].
    ///
    /// # Results
    /// Returns an encrypted [`ProtectedPayload`] bound to the provided context bytes.
    ///
    /// # Errors
    /// * [`VaultError::Encryption`] If the AEAD encryption fails.
    pub fn seal_bytes<K: PayloadKind<C>>(
        &self,
        data: impl AsRef<[u8]>,
        context: &[u8],
    ) -> Result<ProtectedPayload<K, C>, VaultError> {
        let cipher = K::select_cipher(self);
        let bytes = data.as_ref();

        let blob = Self::encrypt_internal(cipher, bytes, context, self.inner.compression)?;
        Ok(ProtectedPayload::from(blob))
    }

    /// Unseals and deserializes a value from `postcard`.
    ///
    /// # Results
    /// Returns the decoded value.
    ///
    /// # Errors
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::PostcardSerialization`] If the decrypted bytes cannot be parsed.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    ///   Unseals and deserializes a value from `postcard`.
    ///
    /// The cryptographic context is taken from [`Tagged::TAG`].
    ///
    /// Prefer [`Vault::unseal_local`] or [`Vault::unseal_fleet`] when you already know
    /// the domain and pass raw sealed bytes directly.
    ///
    /// # Results
    /// Returns the decoded value.
    ///
    /// # Errors
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::PostcardSerialization`] If the decrypted bytes cannot be parsed.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    pub fn unseal<K, T>(&self, payload: impl AsRef<[u8]>) -> Result<T, VaultError>
    where
        K: PayloadKind<C>,
        C: VaultCipher,
        T: VaultSerde,
    {
        let bytes = self.unseal_bytes_raw::<K>(payload.as_ref(), T::TAG.as_bytes())?;
        postcard::from_bytes(&bytes).context("Postcard decoding failed")
    }

    /// Unseals a value from raw bytes using the local domain.
    ///
    /// # Results
    /// Returns the decoded value.
    ///
    /// # Errors
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::PostcardSerialization`] If the decrypted bytes cannot be parsed.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    pub fn unseal_local<T>(&self, payload: impl AsRef<[u8]>) -> Result<T, VaultError>
    where
        T: VaultSerde,
    {
        self.unseal::<Local, T>(payload)
    }

    /// Unseals a value from raw bytes using the fleet domain.
    ///
    /// # Results
    /// Returns the decoded value.
    ///
    /// # Errors
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::PostcardSerialization`] If the decrypted bytes cannot be parsed.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    pub fn unseal_fleet<T>(&self, payload: impl AsRef<[u8]>) -> Result<T, VaultError>
    where
        T: VaultSerde,
    {
        self.unseal::<Fleet, T>(payload)
    }

    /// Decrypts raw sealed bytes or a [`ProtectedPayload`] back into plaintext.
    ///
    /// # Results
    /// Returns the plaintext bytes.
    ///
    /// # Errors
    /// * [`VaultError::InvalidPayload`] If the payload is malformed.
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    pub fn unseal_bytes<K: PayloadKind<C>>(
        &self,
        payload: impl AsRef<[u8]>,
        context: &[u8],
    ) -> Result<Vec<u8>, VaultError> {
        let cipher = K::select_cipher(self);
        Self::decrypt_internal(cipher, payload.as_ref(), context)
    }

    /// Decrypts sealed bytes using the local domain.
    ///
    /// # Results
    /// Returns the plaintext bytes.
    ///
    /// # Errors
    /// * [`VaultError::InvalidPayload`] If the payload is malformed.
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    pub fn unseal_local_bytes(
        &self,
        payload: impl AsRef<[u8]>,
        context: &[u8],
    ) -> Result<Vec<u8>, VaultError> {
        self.unseal_bytes::<Local>(payload, context)
    }

    /// Decrypts sealed bytes using the fleet domain.
    ///
    /// # Results
    /// Returns the plaintext bytes.
    ///
    /// # Errors
    /// * [`VaultError::InvalidPayload`] If the payload is malformed.
    /// * [`VaultError::Decryption`] If the context, key, or data is invalid.
    /// * [`VaultError::Decompression`] If the LZ4 stream is corrupt.
    pub fn unseal_fleet_bytes(
        &self,
        payload: impl AsRef<[u8]>,
        context: &[u8],
    ) -> Result<Vec<u8>, VaultError> {
        self.unseal_bytes::<Fleet>(payload, context)
    }

    fn unseal_bytes_raw<K: PayloadKind<C>>(
        &self,
        payload: &[u8],
        context: &[u8],
    ) -> Result<Vec<u8>, VaultError> {
        let cipher = K::select_cipher(self);
        Self::decrypt_internal(cipher, payload, context)
    }

    fn encrypt_internal(
        cipher: &C,
        data: &[u8],
        aad: &[u8],
        compress: bool,
    ) -> Result<Vec<u8>, VaultError> {
        // Compression is performed BEFORE encryption. This can leak information via ciphertext length
        // in attacker-controlled scenarios. See crate-level documentation for guidance.
        let owned = if compress { lz4_flex::compress_prepend_size(data) } else { Vec::new() };
        let data = if compress { owned.as_slice() } else { data };
        let flags = if compress { FLAG_COMPRESSED } else { 0 };

        let nonce = Self::next_nonce();

        let mut buf = Vec::with_capacity(HEADER_LEN + NONCE_LEN + data.len() + TAG_LEN);
        buf.push(PAYLOAD_VERSION_V1);
        buf.push(flags);
        buf.extend_from_slice(&nonce);
        buf.extend_from_slice(data);

        let (_hdr, rest) = buf.split_at_mut(HEADER_LEN);
        let (_nonce_part, data_part) = rest.split_at_mut(nonce.len());
        let in_out = InOutBuf::from(data_part);

        let tag = cipher.encrypt_inout_detached(&nonce, aad, in_out).map_err(|_| {
            VaultError::Encryption {
                message: "Encryption failed".into(),
                context: Some("AEAD encryption failed".into()),
            }
        })?;

        buf.extend_from_slice(tag.as_slice());
        Ok(buf)
    }

    fn decrypt_internal(cipher: &C, blob: &[u8], aad: &[u8]) -> Result<Vec<u8>, VaultError> {
        if blob.len() < (HEADER_LEN + NONCE_LEN + TAG_LEN) {
            return Err(VaultError::InvalidPayload {
                message: format!(
                    "Payload too short ({} bytes). Expected at least {} bytes",
                    blob.len(),
                    HEADER_LEN + NONCE_LEN + TAG_LEN
                )
                .into(),
                context: None,
            });
        }

        let version = blob[0];
        let flags = blob[1];

        if version != PAYLOAD_VERSION_V1 {
            return Err(VaultError::InvalidPayload {
                message: "Unsupported payload version".into(),
                context: Some(format!("version={version}").into()),
            });
        }

        let rest = &blob[HEADER_LEN..];
        let (nonce_slice, rest) = rest.split_at(NONCE_LEN);
        let (ciphertext, tag_slice) = rest.split_at(rest.len() - TAG_LEN);

        let nonce = nonce_slice.try_into().map_err(|_| VaultError::Decryption {
            message: "Invalid nonce length".into(),
            context: None,
        })?;

        let tag = tag_slice.try_into().map_err(|_| VaultError::Decryption {
            message: "Invalid tag length".into(),
            context: None,
        })?;

        let mut buf = ciphertext.to_vec();
        let in_out = InOutBuf::from(&mut buf[..]);

        cipher.decrypt_inout_detached(&nonce, aad, in_out, &tag).map_err(|_| {
            VaultError::Decryption {
                message: "Decryption failed".into(),
                context: Some("AEAD authentication failed".into()),
            }
        })?;

        let compressed = (flags & FLAG_COMPRESSED) != 0;
        if compressed {
            buf = lz4_flex::decompress_size_prepended(&buf).map_err(|_| {
                VaultError::Decompression {
                    message: "Decompression failed".into(),
                    context: Some("LZ4 stream invalid".into()),
                }
            })?;
        }

        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_vault_builder() {
        let builder =
            Vault::<ChaCha>::builder().derived_keys("master", "salt", "id").unwrap().build();
        assert!(builder.is_ok(), "Vault should build with derived keys");
    }

    #[test]
    fn test_nonce_sequence() {
        let n1 = Vault::<ChaCha>::next_nonce();
        let n2 = Vault::<ChaCha>::next_nonce();

        assert_ne!(n1, n2);
    }

    fn setup_vault(compression: bool) -> Vault<ChaCha> {
        Vault::builder()
            .compression(compression)
            .derived_keys("ikm", "salt", "id")
            .unwrap()
            .build()
            .expect("Vault should build with derived keys")
    }

    #[test]
    fn test_seal_unseal_bytes_local() {
        let vault = setup_vault(false);
        let data = b"sensitive local data";
        let context = b"request-id-456";

        let sealed = vault.seal_bytes::<Local>(data, context).unwrap();
        let unsealed = vault.unseal_bytes::<Local>(&sealed, context).unwrap();

        assert_eq!(data.as_slice(), unsealed.as_slice());
    }

    #[test]
    fn test_seal_unseal_bytes_with_compression() {
        let vault = setup_vault(true);
        let data = b"sensitive local data";
        let context = b"request-id-456";

        let sealed = vault.seal_bytes::<Local>(data, context).unwrap();
        let unsealed = vault.unseal_bytes::<Local>(&sealed, context).unwrap();

        assert_eq!(data.as_slice(), unsealed.as_slice());
    }

    #[test]
    fn test_unseal_fails_with_wrong_context() {
        let vault = setup_vault(false);
        let sealed = vault.seal_bytes::<Local>(b"data", b"correct-context").unwrap();

        let result = vault.unseal_bytes::<Local>(&sealed, b"wrong-context");
        assert!(result.is_err(), "Decryption should fail if AAD/context mismatch");
    }
}
