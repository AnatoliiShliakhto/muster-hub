pub mod fixtures;

use fixtures::*;
use mhub_vault::VaultError;
use mhub_vault::prelude::*;

#[test]
fn test_vault_ext_roundtrip() {
    let vault = setup_vault();
    let config = SecureConfig { db_password: "super-secret".into(), api_key: "abc-123".into() };

    let sealed = config.seal_local(&vault).expect("Sealing failed");
    let unsealed: SecureConfig = vault.unseal_local(&sealed).expect("Unsealing failed");
    assert_eq!(config, unsealed);

    let sealed = config.seal_fleet(&vault).expect("Sealing failed");
    let unsealed: SecureConfig = vault.unseal_fleet(&sealed).expect("Unsealing failed");
    assert_eq!(config, unsealed);
}

#[test]
fn test_context_binding_security() {
    let vault = setup_vault();
    let data = "bound-data".to_owned();
    let context = b"right-context";

    let sealed = vault.seal_bytes::<Local>(data.as_bytes(), context).unwrap();

    // Attempt to unseal with the wrong context
    let result = vault.unseal_bytes::<Local>(&sealed, b"wrong-context");

    assert!(
        matches!(result, Err(VaultError::Decryption { .. })),
        "Must fail with DecryptionFailed when context is wrong"
    );
}

#[test]
fn test_algorithm_agility_cha_cha() {
    let vault =
        Vault::<ChaCha>::builder().derived_keys("key", "salt", "id").unwrap().build().unwrap();

    let data = vec![1, 2, 3, 4, 5];
    let sealed = vault.seal_bytes::<Local>(data.clone(), b"test").unwrap();
    let unsealed = vault.unseal_bytes::<Local>(&sealed, b"test").unwrap();

    assert_eq!(data, unsealed);
}

#[test]
fn test_invalid_context_fails_for_fleet_payloads() {
    let vault = setup_vault();
    let data = "fleet-bound".to_owned();
    let sealed = vault.seal_bytes::<Fleet>(data.as_bytes(), b"fleet").unwrap();

    let result = vault.unseal_bytes::<Fleet>(&sealed, b"local");
    assert!(matches!(result, Err(VaultError::Decryption { .. })));
}
