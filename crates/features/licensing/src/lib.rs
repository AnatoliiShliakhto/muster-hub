//! # Licensing System
//!
//! This crate provides a unified system for license generation and validation. It uses
//! Edwards-curve Digital Signature Algorithm (Ed25519) to ensure that licenses
//! cannot be forged or tampered with.
//!
//! ## Architecture
//!
//! The system is divided into two primary parts:
//!
//! 1.  **Validation ([`validator`]):** Lightweight logic included in production binaries to
//!     verify if a license is authentic and hasn't expired.
//! 2.  **Generation ([`generator`]):** Secure logic used only by the vendor (via `xtask`)
//!     to sign new licenses. Gated behind the `issuance` feature.
//!
//! ## Features
//!
//! * **Cryptographic Security**: Ed25519 signatures via the `ed25519-dalek` crate.
//! * **Machine Binding**: Licenses can be bound to specific hardware IDs or issued as site licenses.
//! * **Feature Flags**: Uses bitflags to define which features are unlocked by a specific license.
//! * **Serialization**: Licenses are serialized to JSON with Base64 encoding for cryptographic bytes.

pub mod constraints;
mod error;
#[cfg(feature = "issuance")]
pub mod generator;
pub mod validator;

pub use crate::error::{LicenseError, LicenseErrorExt};
use mhub_domain::features::FeatureSet;
use serde::{Deserialize, Serialize};

/// A container for a license payload and its corresponding cryptographic signature.
///
/// This structure is typically stored as a JSON file and provided to the end-user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedLicense {
    /// The actual license information (customer, expiry, constraints).
    pub data: LicenseData,
    /// The Ed25519 signature of the `data` field, encoded as a Base64 string in JSON.
    #[serde(with = "bytes_as_base64")]
    pub signature: Vec<u8>,
}

impl SignedLicense {
    /// Serializes the signed license into a compact binary format using Postcard.
    ///
    /// This format is used for cryptographic signature verification and
    /// high-efficiency storage.
    ///
    /// # Errors
    /// Returns [`LicenseError::Postcard`] if serialization fails.
    pub fn encode_bin(&self) -> Result<Vec<u8>, LicenseError> {
        postcard::to_stdvec(self).map_err(LicenseError::from)
    }

    /// Deserializes a signed license from a binary buffer.
    ///
    /// # Errors
    /// Returns [`LicenseError::Postcard`] if the buffer is corrupted or invalid.
    pub fn decode_bin(bytes: &[u8]) -> Result<Self, LicenseError> {
        postcard::from_bytes(bytes).map_err(LicenseError::from)
    }

    /// Serializes the signed license into a human-readable JSON string.
    ///
    /// This format is suitable for configuration files or transmission over
    /// text-based protocols. Note that cryptographic fields (signature/salt)
    /// are encoded as Base64.
    ///
    /// # Errors
    /// Returns [`LicenseError::Serialize`] if serialization fails.
    pub fn to_json(&self) -> Result<String, LicenseError> {
        serde_json::to_string(self).map_err(LicenseError::from)
    }

    /// Deserializes a signed license from a JSON string.
    ///
    /// # Errors
    /// Returns [`LicenseError::Serialize`] if the JSON is malformed or
    /// contains invalid Base64 data.
    pub fn from_json(json: &str) -> Result<Self, LicenseError> {
        serde_json::from_str(json).map_err(LicenseError::from)
    }

    /// Validates a signed license against the provided public key.
    ///
    /// This is the primary entry point for license verification. It performs both
    /// expiration and cryptographic signature checks.
    ///
    /// # Arguments
    /// * `key` - A 32-byte array representing the trusted public key for verification.
    ///
    /// # Returns
    /// * `Ok(())` if the license is authentic and valid.
    ///
    /// # Errors
    /// * [`LicenseError::Expired`] if the current system time is past the `expires_at` timestamp.
    /// * [`LicenseError::InvalidSignature`] (via `ed25519_dalek`) if the data has been tampered with.
    /// * [`LicenseError::Internal`] if the system clock cannot be accessed.
    pub fn validate(&self, key: &[u8; 32]) -> Result<(), LicenseError> {
        validator::validate_license(self, key)
    }

    /// Securely wipes the license data from memory and consumes the instance.
    ///
    /// Use this method when you are finished processing a license to ensure
    /// that sensitive customer information and salts do not persist in RAM
    /// longer than necessary.
    pub fn secure_clear(mut self) {
        use zeroize::Zeroize;
        self.data.customer.zeroize();
        self.data.alias.zeroize();
        self.data.salt.zeroize();
        self.signature.zeroize();
    }
}

/// The core data payload of a license.
///
/// This structure defines what the user is allowed to do and for how long.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LicenseData {
    /// The name of the licensed entity/customer.
    pub customer: String,
    /// Short alias used for namespaces or resource naming.
    pub alias: String,
    /// Hardware constraints defining where this license can run.
    pub constraint: MachineConstraint,
    /// A bitset of enabled feature flags.
    pub features: FeatureSet,
    /// A unique salt to prevent identical licenses from having the same signature.
    #[serde(with = "bytes_as_base64")]
    pub salt: Vec<u8>,
    /// UNIX timestamp when the license was created.
    pub issued: i64,
    /// UNIX timestamp (in seconds) indicating when the license expires.
    pub expires: i64,
}

/// Defines hardware binding rules for a license.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MachineConstraint {
    /// The license is not bound to any specific hardware (Site License).
    Any,
    /// The license is valid only if at least `min_matches` of the provided IDs are found.
    /// This provides "Fuzzy" matching to survive minor hardware changes.
    Threshold {
        /// The list of valid hardware fingerprints.
        ids: Vec<String>,
        /// IDs from the list must match for the license to be valid.
        min_matches: u16,
    },
}

/// Helper module for transparently serializing byte buffers to Base64 strings.
#[allow(clippy::redundant_pub_crate)]
pub mod bytes_as_base64 {
    use base64::{Engine as _, engine::general_purpose};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes a byte vector into a URL-safe Base64 string without padding.
    pub(super) fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let mut buf = String::with_capacity((v.len() * 4).div_ceil(3));
        general_purpose::STANDARD_NO_PAD.encode_string(v, &mut buf);
        String::serialize(&buf, s)
    }

    /// Deserializes a Base64 string back into a byte vector.
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        match general_purpose::STANDARD_NO_PAD.decode(String::deserialize(d)?) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(serde::de::Error::custom(format!("Invalid Base64: {e}"))),
        }
    }
}
