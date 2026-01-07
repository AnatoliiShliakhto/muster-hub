//! # License Validation Module
//!
//! This module provides functions to verify the authenticity and validity of issued licenses.
//! It is designed to be lightweight and included in all production builds (client and server).
//!
//! ## Validation Logic
//! The validation process follows two strict steps:
//! 1. **Temporal Check**: Ensures the license has not expired against the current system time.
//! 2. **Cryptographic Check**: Verifies that the license data was signed by the official
//!    *Master Private Key* using the corresponding public key.

use crate::constraints::{generate_machine_id_compound, parse_machine_id_compound};
use crate::error::{LicenseError, LicenseErrorExt};
use crate::{MachineConstraint, SignedLicense};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use fxhash::FxHashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Validates a signed license against the provided public key.
///
/// This is the primary entry point for license verification. It performs both
/// expiration and cryptographic signature checks.
///
/// # Arguments
/// * `license` - A reference to the [`SignedLicense`] structure containing the data and signature.
/// * `key` - A 32-byte array representing the trusted public key for verification.
///
/// # Returns
/// * `Ok(())` if the license is authentic and valid.
///
/// # Errors
/// * [`LicenseError::Expired`] if the current system time is past the `expires_at` timestamp.
/// * [`LicenseError::InvalidSignature`] (via `ed25519_dalek`) if the data has been tampered with.
/// * [`LicenseError::Internal`] if the system clock cannot be accessed.
pub fn validate_license(license: &SignedLicense, key: &[u8; 32]) -> Result<(), LicenseError> {
    // 1. Check expiry
    check_expiry(license)?;

    // 2. Verify signature
    verify_signature(license, key)?;

    Ok(())
}

/// Internal helper to check the license expiration date.
///
/// Compares the current UNIX timestamp with the `expires_at` value stored in the license.
fn check_expiry(license: &SignedLicense) -> Result<(), LicenseError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| LicenseError::Internal {
            message: e.to_string().into(),
            context: Some("Failed to get current system time".into()),
        })?
        .as_secs()
        .cast_signed();

    if now < license.data.issued {
        return Err(LicenseError::Internal {
            message: "System clock is set before license issuance date".into(),
            context: Some("Current time comparison failed".into()),
        });
    }

    if now > license.data.expires {
        return Err(LicenseError::Expired {
            message: format!("License expired at Unix timestamp {}", license.data.expires).into(),
            context: Some("Expiration Check".into()),
        });
    }

    validate_hardware(&license.data.constraint)?;

    Ok(())
}

/// Checks if the current machine satisfies the license hardware constraints.
fn validate_hardware(constraint: &MachineConstraint) -> Result<(), LicenseError> {
    match constraint {
        MachineConstraint::Any => Ok(()),
        MachineConstraint::Threshold { ids, min_matches } => {
            // Current machine: compound -> 3 components
            let current_compound = generate_machine_id_compound()?;
            let current_parts = parse_machine_id_compound(&current_compound)?;
            let current_set: FxHashSet<&str> = current_parts.iter().map(String::as_str).collect();

            // For each allowed machine (compound string), compute how many components match.
            let mut best: u16 = 0;

            for allowed_compound in ids {
                let allowed_parts = parse_machine_id_compound(allowed_compound)?;
                let allowed_set: FxHashSet<&str> =
                    allowed_parts.iter().map(String::as_str).collect();

                let matches = allowed_set.intersection(&current_set).count();
                let matches_u16 = u16::try_from(matches).unwrap_or(u16::MIN);
                best = best.max(matches_u16);

                if best >= *min_matches {
                    return Ok(());
                }
            }

            Err(LicenseError::HardwareMismatch {
                message: format!(
                    "Hardware constraint not satisfied: best match is {best}, expected at least {min_matches}"
                )
                    .into(),
                context: Some("Hardware constraint validation".into()),
            })
        },
    }
}

/// Internal helper to verify the Ed25519 cryptographic signature.
///
/// It reconstructs the signed payload by serializing the [`LicenseData`] and
/// checking it against the signature using the provided public key.
fn verify_signature(license: &SignedLicense, public_key: &[u8; 32]) -> Result<(), LicenseError> {
    let verifying_key = VerifyingKey::from_bytes(public_key)?;
    let signature = Signature::from_slice(&license.signature)?;

    let data_bytes = postcard::to_stdvec(&license.data).context("Binary serialization failed")?;

    verifying_key.verify(&data_bytes, &signature)?;

    Ok(())
}
