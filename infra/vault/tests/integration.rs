use mhub_vault::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct SecureConfig {
    db_password: String,
    api_key: String,
}

fn setup_vault() -> Vault {
    Vault::builder()
        .with_derived_keys("master-secret-123", "unique-salt", "machine-01")
        .with_compression(true)
        .build()
        .expect("Vault setup failed")
}

#[test]
fn test_vault_ext_roundtrip() {
    let vault = setup_vault();
    let config = SecureConfig {
        db_password: "super-secret".into(),
        api_key: "abc-123".into(),
    };

    // 1. Seal using Extension Trait (Local Domain)
    let sealed = config.seal_local(&vault).expect("Sealing failed");

    // 2. Unseal using explicit unseal (requires type name matching)
    let unsealed: SecureConfig = vault
        .unseal_json(&sealed, &"integration::SecureConfig")
        .expect("Unsealing failed");

    assert_eq!(config, unsealed);
}

#[test]
fn test_context_binding_security() {
    let vault = setup_vault();
    let data = "bound-data".to_owned();
    let context = "right-context";

    let sealed = vault.seal_json::<Local>(&data, &context).unwrap();

    // Attempt to unseal with the wrong context
    let result: Result<String, VaultError> = vault.unseal_json(&sealed, &"wrong-context");

    assert!(
        matches!(result, Err(VaultError::DecryptionFailed(_))),
        "Must fail with DecryptionFailed when context is wrong"
    );
}

#[test]
fn test_algorithm_agility_cha_cha() {
    let vault = Vault::<ChaCha>::builder()
        .with_derived_keys("key", "salt", "id")
        .build()
        .unwrap();

    let data = vec![1, 2, 3, 4, 5];
    let sealed = vault.seal_raw::<Local>(data.clone(), &"test").unwrap();
    let unsealed = vault.unseal_raw::<Local>(&sealed, &"test").unwrap();

    assert_eq!(data, unsealed);
}