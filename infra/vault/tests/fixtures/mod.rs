use mhub_vault::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Tagged)]
#[tagged("SecureConfig")]
pub struct SecureConfig {
    pub db_password: String,
    pub api_key: String,
}

pub fn setup_vault() -> Vault {
    Vault::builder()
        .with_derived_keys("master-secret-123", "unique-salt", "machine-01")
        .with_compression(true)
        .build()
        .expect("Vault setup failed")
}
