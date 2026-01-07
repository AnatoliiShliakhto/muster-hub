use anyhow::Result;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use mhub::domain::Features;
use mhub::kernel::system::licensing::{
    LicenseData, MachineConstraint, SignedLicense,
};
use rand::TryRngCore;
use rand::rngs::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_secret() -> Result<[u8; 32]> {
    let mut secret = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut secret)
        .map_err(|_| anyhow::anyhow!("Failed to generate secret"))?;
    Ok(secret)
}

pub fn generate_keypair() -> Result<(SigningKey, VerifyingKey)> {
    let mut seed = [0u8; 32];

    OsRng
        .try_fill_bytes(&mut seed)
        .map_err(|_| anyhow::anyhow!("Failed to generate seed"))?;

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    Ok((signing_key, verifying_key))
}

pub struct UniversalLicenseConfig {
    pub customer: String,
    pub constraint: MachineConstraint,
    pub days_valid: u64,
    pub features: Vec<String>,
    pub salt: [u8; 32],
}

pub fn generate_universal_license(
    private_key_bytes: &[u8; 32],
    config: UniversalLicenseConfig,
) -> Result<SignedLicense> {
    let signing_key = SigningKey::from_bytes(private_key_bytes);

    let expires_at = (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        + (config.days_valid * 24 * 3600))
        .cast_signed();

    let mut features = Features::empty();
    config.features.iter().for_each(|feature| {
        features.insert(Features::from(feature.to_lowercase().as_str()));
    });

    let data = LicenseData {
        customer: config.customer,
        constraint: config.constraint,
        expires_at,
        features,
        salt: config.salt.to_vec(),
    };

    let data_bytes = serde_json::to_vec(&data)?;
    let signature = signing_key.sign(&data_bytes).to_bytes().to_vec();

    Ok(SignedLicense { data, signature })
}
