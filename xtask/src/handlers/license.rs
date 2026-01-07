use crate::error::{AppError, AppErrorExt};
use crate::models::keyset::Keyset;
use mhub_licensing::generator::{
    UniversalLicenseConfig, generate_secret, generate_universal_license,
};
use mhub_licensing::{MachineConstraint, SignedLicense};
use std::fs;

/// Generates a license file.
///
/// # Result
/// Returns an `Ok(())` indicating success or failure of the license generation process.
///
/// # Errors
/// Returns an error if the license generation process encounters any issues, such as invalid input or file operations.
pub fn generate_license(
    customer: &str,
    alias: &str,
    machines: &str,
    min_matches: u16,
    days: u64,
) -> Result<(), AppError> {
    fs::create_dir_all("private/licenses").ok();

    // 1. Parse Machine Constraint
    let constraint = if machines.to_uppercase() == "ANY" {
        MachineConstraint::Any
    } else {
        MachineConstraint::Threshold {
            ids: machines.split(',').map(|s| s.trim().to_owned()).collect(),
            min_matches,
        }
    };

    let salt: [u8; 32] = if let Ok(s) = fs::read(format!("private/licenses/{alias}.lic")) {
        let lic = SignedLicense::decode_bin(&s)?;
        lic.data
            .salt
            .try_into()
            .map_err(|_| AppError::internal().with_context("Invalid key length"))?
    } else {
        generate_secret()?
    };

    // 3. Generate using the universal function
    let config = UniversalLicenseConfig {
        customer: customer.to_owned(),
        alias: alias.to_owned(),
        constraint,
        days,
        salt,
    };

    let keyset =
        fs::read("private/keyset").with_context("Failed to read keyset for license generation")?;
    let keyset: Keyset = postcard::from_bytes(&keyset).map_err(|e| {
        AppError::parse().with_context_fn(|| format!("Failed to deserialize keyset: {e}"))
    })?;

    let signed = generate_universal_license(&keyset.master_key, config)?;

    let bytes = signed.encode_bin()?;
    fs::write(format!("private/licenses/{alias}.lic"), bytes)?;

    println!("✅ License generated successfully for `{}`", signed.data.customer);

    Ok(())
}
