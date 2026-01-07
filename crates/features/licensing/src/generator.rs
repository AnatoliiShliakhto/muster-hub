//! # License Generation Module
//!
//! This module provides the cryptographic routines required to issue and sign licenses.
//! It is strictly gated behind the `licensing-gen` feature to ensure that signing
//! logic and private key handling are not included in client or server production builds.
//!
//! ## Security Warnings
//! * This module handles **Private Keys**. Ensure the environment where this is executed
//!   is secure and that `private/master-key` is never committed to version control.
//! * Uses `Ed25519` for deterministic, high-security digital signatures.

use crate::error::LicenseError;
use crate::{LicenseData, MachineConstraint, SignedLicense};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use getrandom::fill;
use mhub_domain::features::FeatureSet;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

/// Generates a high-entropy 32-byte secret using the operating system's Cryptographically
/// Secure Pseudo-Random Number Generator (CSPRNG).
///
/// This function is primarily used to generate unique salts for licenses. By incorporating a unique salt
/// into each license payload, we ensure that two identical licenses (same customer, features, and
/// expiration) will result in completely different cryptographic signatures. This prevents replay
/// attacks and pattern analysis of issued licenses.
///
/// # Returns
/// * `Ok([u8; 32])` - A cryptographically strong 256-bit random array.
/// * `Err(LicenseError)` - If the operating system's entropy source fails.
///
/// # Errors
/// This function returns [`LicenseError::Internal`] if the underlying system RNG fails to fill
/// the buffer. This is a rare critical failure that usually indicates an environment-level issue
/// with the OS entropy pool.
///
/// # Security
/// This function uses the system RNG (via `getrandom`), which maps to the best available source
/// of randomness on the platform (e.g., `getrandom` on Linux, `BCryptGenRandom` on Windows).
pub fn generate_secret() -> Result<[u8; 32], LicenseError> {
    let mut secret = [0u8; 32];
    fill(&mut secret).map_err(|e| LicenseError::Internal {
        message: e.to_string().into(),
        context: Some("Failed to generate secret".into()),
    })?;
    Ok(secret)
}

/// Creates a new Ed25519 keypair for high-security license signing and validation.
///
/// This function generates a 32-byte seed from the system's entropy source and derives
/// a standard Edwards-curve Digital Signature Algorithm (Ed25519) keypair.
///
/// # Lifecycle & Security
/// * **Private Key ([`SigningKey`]):** This must be kept strictly confidential. It is used
///   by the license issuer (server/admin tools) to sign license payloads. If compromised,
///   an attacker could forge valid licenses.
/// * **Public Key ([`VerifyingKey`]):** This is distributed with your application. It is
///   used by the client or server to validate that a license was indeed issued by you.
/// * **Memory Safety:** The temporary seed used for generation is explicitly zeroed out
///   after the keypair is created to prevent sensitive data from lingering in RAM.
///
/// # Returns
/// * `Ok((SigningKey, VerifyingKey))` - The generated cryptographic keypair.
/// * `Err(LicenseError)` - If the system's random number generator is unavailable.
///
/// # Errors
/// Returns [`LicenseError::Internal`] if the system RNG fails to provide sufficient entropy.
pub fn generate_keypair() -> Result<(SigningKey, VerifyingKey), LicenseError> {
    let mut seed = [0u8; 32];

    fill(&mut seed).map_err(|e| LicenseError::Internal {
        message: e.to_string().into(),
        context: Some("Failed to generate seed".into()),
    })?;

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    seed.zeroize();

    Ok((signing_key, verifying_key))
}

/// Input configuration for the universal license factory.
#[derive(Debug)]
pub struct UniversalLicenseConfig {
    /// The name of the customer or entity.
    pub customer: String,
    /// Short alias used for namespaces or resource naming.
    pub alias: String,
    /// Hardware binding requirements (e.g., Specific IDs or 'Any').
    pub constraint: MachineConstraint,
    /// How many days from the moment of generation the license remains valid.
    pub days: u64,
    /// List of feature slugs to enable (e.g., `["quiz", "survey"]`).
    pub features: Vec<String>,
    /// Unique salt for this specific license.
    pub salt: [u8; 32],
}

/// The core factory function that produces a cryptographically signed license.
///
/// This function:
/// 1. Calculates the expiration timestamp based on `days_valid`.
/// 2. Maps string-based feature slugs to the internal [`Features`] bitflags.
/// 3. Serializes the payload to JSON.
/// 4. Signs the payload using the provided 32-byte Ed25519 private key.
///
/// # Errors
/// Returns an error if the private key is invalid, time calculation fails,
/// or serialization encounters an issue.
pub fn generate_universal_license(
    private_key: &[u8; 32],
    config: UniversalLicenseConfig,
) -> Result<SignedLicense, LicenseError> {
    let signing_key = SigningKey::from_bytes(private_key);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| LicenseError::Internal {
            message: e.to_string().into(),
            context: Some("Failed to calculate current time".into()),
        })?
        .as_secs();
    let issued = now.cast_signed();
    let expires = (now + (config.days * 24 * 3600)).cast_signed();

    let mut features = FeatureSet::empty();
    config.features.iter().for_each(|feature| {
        features.insert(FeatureSet::from(feature.to_lowercase().as_str()));
    });

    let data = LicenseData {
        customer: config.customer,
        alias: config.alias,
        constraint: config.constraint,
        issued,
        expires,
        features,
        salt: config.salt.to_vec(),
    };

    let bytes = postcard::to_stdvec(&data)?;
    let signature = signing_key.sign(&bytes).to_bytes().to_vec();

    Ok(SignedLicense { data, signature })
}
