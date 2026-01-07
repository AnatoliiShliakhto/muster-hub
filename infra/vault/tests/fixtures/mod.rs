use mhub_vault::prelude::*;
#[vault_model(tag = "SecureConfig")]
#[derive(Clone)]
pub struct SecureConfig {
    pub db_password: String,
    pub api_key: String,
}

/// Initializes a Vault instance with predefined keys and settings for testing.
/// # Panics
/// * If Vault setup fails, the function will panic.
#[must_use]
pub fn setup_vault() -> Vault {
    Vault::builder()
        .derived_keys("master-secret-123", "unique-salt", "machine-01")
        .unwrap()
        .compression(true)
        .build()
        .expect("Vault setup failed")
}
