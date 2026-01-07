use crate::models::keyset::Keyset;
use mhub_licensing::generator::{
    UniversalLicenseConfig, generate_secret, generate_universal_license,
};
use mhub_licensing::{MachineConstraint, SignedLicense};
use std::fs;

pub fn generate_license(
    customer: &str,
    machines: &str,
    min_matches: u16,
    features: &str,
    days: u64,
) -> anyhow::Result<()> {
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

    // 2. Parse Features
    let feature_list = features.split(',').map(|s| s.trim().to_owned()).collect();

    let salt: [u8; 32] = if let Ok(s) = fs::read(format!("private/licenses/{customer}.lic")) {
        let lic = SignedLicense::decode_bin(&s)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize license file: {e}"))?;
        lic.data.salt.try_into().map_err(|_| anyhow::anyhow!("Invalid key length"))?
    } else {
        generate_secret()?
    };

    // 3. Generate using the universal function
    let config = UniversalLicenseConfig {
        customer: customer.to_owned(),
        constraint,
        days_valid: days,
        features: feature_list,
        salt,
    };

    let keyset_bytes =
        fs::read("private/keyset").map_err(|e| anyhow::anyhow!("Failed to read keyset: {e}"))?;
    let keyset: Keyset = postcard::from_bytes(&keyset_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize keyset: {e}"))?;

    let signed = generate_universal_license(&keyset.master_key, config)?;

    let bytes = signed.encode_bin()?;
    fs::write(format!("private/licenses/{customer}.lic"), bytes)?;

    println!("✅ License generated successfully for {}", signed.data.customer);

    Ok(())
}
