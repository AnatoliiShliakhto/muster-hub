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

use crate::constraints::generate_machine_id;
use crate::{LicenseError, MachineConstraint, Result, SignedLicense, LicenseErrorExt};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::time::{SystemTime, UNIX_EPOCH};

/// Validates a signed license against the provided public key.
///
/// This is the primary entry point for license verification. It performs both
/// expiration and cryptographic signature checks.
///
/// # Arguments
/// * `license` - A reference to the [`SignedLicense`] structure containing the data and signature.
/// * `public_key` - A 32-byte array representing the trusted public key for verification.
///
/// # Returns
/// * `Ok(())` if the license is authentic and valid.
///
/// # Errors
/// * [`LicenseError::Expired`] if the current system time is past the `expires_at` timestamp.
/// * [`LicenseError::InvalidSignature`] (via `ed25519_dalek`) if the data has been tampered with.
/// * [`LicenseError::Internal`] if the system clock cannot be accessed.
pub fn validate_license(license: &SignedLicense, public_key: &[u8; 32]) -> Result<()> {
    // 1. Check expiry
    check_expiry(license)?;

    // 2. Verify signature
    verify_signature(license, public_key)?;

    Ok(())
}

/// Internal helper to check the license expiration date.
///
/// Compares the current UNIX timestamp with the `expires_at` value stored in the license.
fn check_expiry(license: &SignedLicense) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| LicenseError::Internal {
            message: e.to_string().into(),
            context: Some("Failed to get current system time".into()),
        })?
        .as_secs()
        .cast_signed();

    if now < license.data.issued_at {
        return Err(
            LicenseError::Internal {
                message: "System clock is set before license issuance date".into(),
                context: Some("Current time comparison failed".into()),
            });
    }

    if now > license.data.expires_at {
        return Err(LicenseError::Expired {
            message: format!(
                "License expired at Unix timestamp {}",
                license.data.expires_at
            ).into(),
            context: Some("Expiration Check".into()),
        });
    }

    validate_hardware(&license.data.constraint)?;

    Ok(())
}

/// Checks if the current machine satisfies the license hardware constraints.
fn validate_hardware(constraint: &MachineConstraint) -> Result<()> {
    match constraint {
        MachineConstraint::Any => Ok(()),
        MachineConstraint::Threshold { ids, min_matches } => {
            let current_id = generate_machine_id()?;

            let matches =
                u16::try_from(ids.iter().filter(|&allowed_id| allowed_id == &current_id).count())
                    .unwrap_or(1);

            if matches >= *min_matches { Ok(()) } else {
                Err(LicenseError::HardwareMismatch {
                    message: format!("Found {matches} matches, expected at least {min_matches}").into(),
                    context: Some("Hardware constraint validation".into()),
                })
            }
        },
    }
}

/// Internal helper to verify the Ed25519 cryptographic signature.
///
/// It reconstructs the signed payload by serializing the [`LicenseData`] and
/// checking it against the signature using the provided public key.
fn verify_signature(license: &SignedLicense, public_key_bytes: &[u8; 32]) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)?;
    let signature = Signature::from_slice(&license.signature)?;

    let data_bytes = postcard::to_stdvec(&license.data)
        .with_context("Binary serialization failed")?;

    verifying_key.verify(&data_bytes, &signature)?;

    Ok(())
}
