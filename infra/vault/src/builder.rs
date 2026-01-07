use crate::engine::{Vault, VaultInner};
use crate::error::VaultError;
use crate::types::{Aes, VaultCipher};
use aead::Key;
use hkdf::Hkdf;
use private::Sealed;
use sha2::Sha256;
use std::marker::PhantomData;
use std::sync::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Default, ZeroizeOnDrop)]
pub struct NoKeys;
#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct WithKeys {
    local: [u8; 32],
    fleet: [u8; 32],
}

mod private {
    pub(super) trait Sealed {}
}
impl Sealed for NoKeys {}
impl Sealed for WithKeys {}

/// A builder for secure initialization of the [`Vault`].
///
/// Implements `ZeroizeOnDrop` to ensure that raw key material is cleared from
/// memory as soon as the builder is no longer needed.
#[allow(private_bounds)]
#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct VaultBuilder<C: VaultCipher = Aes, K: Sealed + ZeroizeOnDrop = NoKeys> {
    #[zeroize(skip)]
    _cipher: PhantomData<C>,
    compression: bool,
    keys: K,
}

impl<C: VaultCipher> Default for VaultBuilder<C> {
    fn default() -> Self {
        Self { _cipher: PhantomData, compression: false, keys: NoKeys }
    }
}

impl<C: VaultCipher> VaultBuilder<C> {
    /// Creates a new empty builder.
    ///
    /// # Results
    /// Returns a fresh [`VaultBuilder`] with compression disabled.
    ///
    /// # Errors
    /// None.
    #[must_use = "Builder must be configured with `derived_keys` before use"]
    pub fn new() -> Self {
        Self::default()
    }

    /// Derives cryptographic keys using HKDF-SHA256.
    ///
    /// # Arguments
    /// * `ikm`: Input Keying Material (Master Password/Secret).
    /// * `salt`: Uniquifies keys across different environments.
    /// * `id`: Binds the [`Local`] key to a specific machine/identity.
    ///
    /// # Results
    /// Returns a [`VaultBuilder`] configured with derived local and fleet keys.
    ///
    /// # Errors
    /// Returns [`VaultError::InvalidConfiguration`] if key derivation fails.
    pub fn derived_keys(
        self,
        ikm: impl AsRef<[u8]>,
        salt: impl AsRef<[u8]>,
        id: impl AsRef<[u8]>,
    ) -> Result<VaultBuilder<C, WithKeys>, VaultError> {
        let (_, hk) = Hkdf::<Sha256>::extract(Some(salt.as_ref()), ikm.as_ref());
        let mut fleet = [0u8; 32];
        let mut local = [0u8; 32];

        hk.expand(b"v1_fleet:", &mut fleet).map_err(|_| VaultError::Encryption {
            message: "HKDF expansion failed for fleet key".into(),
            context: None,
        })?;

        let mut info = Vec::from(b"v1_local:");
        info.extend_from_slice(id.as_ref());

        hk.expand(&info, &mut local).map_err(|_| VaultError::Encryption {
            message: "HKDF expansion failed for local key".into(),
            context: None,
        })?;

        info.zeroize();

        Ok(VaultBuilder {
            _cipher: PhantomData,
            compression: self.compression,
            keys: WithKeys { local, fleet },
        })
    }
}

#[allow(private_bounds)]
impl<C: VaultCipher, K: Sealed + ZeroizeOnDrop> VaultBuilder<C, K> {
    /// Toggles LZ4 compression for sealed payloads by default.
    ///
    /// # Security / Threat Model
    /// Compression is applied **before encryption**. While this is the correct order for
    /// AEAD usage, it may leak information via ciphertext length when attacker-controlled
    /// data is sealed and the attacker can observe ciphertext sizes.
    ///
    /// Recommended:
    /// - Enable compression for internal storage where the payload length is not attacker-observable.
    /// - Disable compression for attacker-controlled inputs or public protocols.
    ///
    /// Compression state is stored in the payload header for safe unsealing.
    ///
    /// # Results
    /// Returns the builder with compression set to the provided value.
    ///
    /// # Errors
    /// None.
    #[must_use]
    pub const fn compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }
}

impl<C: VaultCipher> VaultBuilder<C, WithKeys> {
    /// Finalizes vault construction and `zeroes` the builder.
    ///
    /// # Results
    /// Returns a fully initialized [`Vault`].
    ///
    /// # Errors
    /// Returns [`VaultError::InvalidConfiguration`] if keys were not provided or derived.
    pub fn build(mut self) -> Result<Vault<C>, VaultError> {
        let vault = VaultInner {
            local_cipher: Self::init_cipher(&self.keys.local, "Local")?,
            fleet_cipher: Self::init_cipher(&self.keys.fleet, "Fleet")?,
            compression: self.compression,
        };

        self.zeroize();

        Ok(Vault { inner: Arc::new(vault) })
    }

    fn init_cipher(key: &[u8; 32], context: &'static str) -> Result<C, VaultError> {
        let key = Key::<C>::try_from(&key[..]).map_err(|_| VaultError::InvalidConfiguration {
            message: format!("Invalid key length {}, must be 32 bytes", key.len()).into(),
            context: Some(context.into()),
        })?;
        Ok(C::new(&key))
    }
}
