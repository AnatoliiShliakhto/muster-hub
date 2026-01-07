mod fixtures;

use mhub_vault::prelude::*;
use fixtures::*;

#[test]
fn test_vault_ext_roundtrip() {
    let vault = setup_vault();
    let config = SecureConfig {
        db_password: "super-secret".into(),
        api_key: "abc-123".into(),
    };

    let sealed =
        config.seal_local_with_ctx(&vault, b"").expect("Sealing failed");
    let unsealed: SecureConfig =
        vault.unseal(&sealed).expect("Unsealing failed");
    assert_eq!(config, unsealed);

    let sealed = config.seal_local_tagged(&vault).expect("Sealing failed");
    let unsealed: SecureConfig = vault
        .unseal_with_ctx(&sealed, &"SecureConfig")
        .expect("Unsealing failed");
    assert_eq!(config, unsealed);

    let sealed = config.seal_fleet_tagged(&vault).expect("Sealing failed");
    let unsealed: SecureConfig =
        vault.unseal_tagged(&sealed).expect("Unsealing failed");
    assert_eq!(config, unsealed);

    let sealed = config.seal_fleet(&vault).expect("Sealing failed");
    let unsealed: SecureConfig =
        vault.unseal(&sealed).expect("Unsealing failed");
    assert_eq!(config, unsealed);

    let sealed =
        config.seal_fleet_with_ctx(&vault, &b"").expect("Sealing failed");
    let unsealed: SecureConfig =
        vault.unseal(&sealed).expect("Unsealing failed");
    assert_eq!(config, unsealed);
}

#[test]
fn test_context_binding_security() {
    let vault = setup_vault();
    let data = "bound-data".to_owned();
    let context = "right-context";

    let sealed = vault.seal_json::<Local>(&data, &context).unwrap();

    // Attempt to unseal with the wrong context
    let result: Result<String, VaultError> =
        vault.unseal_json(&sealed, &"wrong-context");

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
