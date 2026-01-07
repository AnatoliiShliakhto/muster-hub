use mhub_domain::licensing::MachineConstraint;
use crate::services::licensing::UniversalLicenseConfig;

fn generate_license(customer: String, machines: String, features: String, days: u64) -> anyhow::Result<()> {
    // 1. Parse Machine Constraint
    let constraint = if machines.to_uppercase() == "ANY" {
        MachineConstraint::Any
    } else if machines.contains(',') {
        MachineConstraint::List(machines.split(',').map(|s| s.trim().to_string()).collect())
    } else {
        MachineConstraint::Single(machines.trim().to_string())
    };

    // 2. Parse Features
    let feature_list = features.split(',').map(|s| s.trim().to_string()).collect();

    // 3. Generate using the universal function
    let config = UniversalLicenseConfig {
        customer,
        constraint,
        days_valid: days,
        features: feature_list,
    };

    // (Key loading and signing logic from previous step...)
    // let signed = generate_universal_license(&key_bytes, config)?;

    Ok(())
}