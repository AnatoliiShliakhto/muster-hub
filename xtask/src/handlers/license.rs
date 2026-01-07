use crate::services::licensing::{
    UniversalLicenseConfig, generate_secret, generate_universal_license,
};
use mhub::kernel::system::licensing::{MachineConstraint, SignedLicense};
use std::fs;

pub fn generate_license(
    customer: &str,
    machines: &str,
    features: &str,
    days: u64,
) -> anyhow::Result<()> {
    fs::create_dir_all("private/licenses").ok();

    // 1. Parse Machine Constraint
    let constraint = if machines.to_uppercase() == "ANY" {
        MachineConstraint::Any
    } else {
        MachineConstraint::MachineIds(
            machines.split(',').map(|s| s.trim().to_owned()).collect(),
        )
    };

    // 2. Parse Features
    let feature_list =
        features.split(',').map(|s| s.trim().to_owned()).collect();

    let salt: [u8; 32] = if let Ok(s) =
        fs::read(format!("private/licenses/{customer}.lic"))
    {
        let lic = serde_json::from_slice::<SignedLicense>(&s).map_err(|e| {
            anyhow::anyhow!("Failed to deserialize license file: {e}")
        })?;
        lic.data
            .salt
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?
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

    let key_bytes = fs::read("private/master-key").map_err(|_| {
        anyhow::anyhow!("Private key not found at private/master-key")
    })?[..32]
        .try_into()?;

    let signed = generate_universal_license(&key_bytes, config)?;

    let json = serde_json::to_string(&signed)?;
    fs::write(format!("private/licenses/{customer}.lic"), json)?;

    println!("✅ License generated successfully for {}", signed.data.customer);

    Ok(())
}
