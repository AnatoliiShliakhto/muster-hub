//! License validation utilities

use super::LicenseError;
use super::types::SignedLicense;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::time::{SystemTime, UNIX_EPOCH};

/// Validates a signed license
pub fn validate_license(
    license: &SignedLicense,
    public_key_bytes: &[u8; 32],
) -> Result<(), LicenseError> {
    // 1. Check expiry
    check_expiry(license)?;

    // 2. Verify signature
    verify_signature(license, public_key_bytes)?;

    Ok(())
}

fn check_expiry(license: &SignedLicense) -> Result<(), LicenseError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| LicenseError::Internal(e.to_string()))?
        .as_secs()
        .cast_signed();

    if now > license.data.expires_at {
        Err(LicenseError::Expired)?;
    }

    Ok(())
}

fn verify_signature(
    license: &SignedLicense,
    public_key_bytes: &[u8; 32],
) -> Result<(), LicenseError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)?;
    let signature = Signature::from_slice(&license.signature)?;
    let data_bytes = serde_json::to_vec(&license.data)?;

    verifying_key.verify(&data_bytes, &signature)?;

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::system::licensing::{
//         crypto::generate_keypair,
//         generator::{generate_universal_license, UniversalLicenseConfig},
//         types::MachineConstraint,
//     };
//
//     #[test]
//     fn test_valid_license() {
//         let (signing_key, verifying_key) = generate_keypair().unwrap();
//
//         let config = UniversalLicenseConfig {
//             customer: "Test".into(),
//             constraint: MachineConstraint::Unlimited,
//             days_valid: 365,
//             features: vec!["auth".into()],
//             secret: [1
