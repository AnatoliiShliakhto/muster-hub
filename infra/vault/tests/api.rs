pub mod fixtures;

use fixtures::setup_vault;
use mhub_vault::prelude::*;

#[vault_model(tag = "v1.profile")]
struct Profile {
    username: String,
    enabled: bool,
}

#[test]
fn seal_unseal_postcard_with_context_roundtrip() {
    let vault = setup_vault();
    let profile = Profile { username: "ada".to_owned(), enabled: true };

    let sealed = vault.seal::<Local, _>(&profile).expect("seal failed");
    let unsealed: Profile = vault.unseal_local(&sealed).expect("unseal failed");

    assert_eq!(profile, unsealed);
}

#[test]
fn seal_unseal_bytes_roundtrip() {
    let vault = setup_vault();
    let payload = b"byte-oriented payload";

    let sealed = vault.seal_bytes::<Local>(payload, b"bytes").expect("seal failed");
    let unsealed = vault.unseal_bytes::<Local>(&sealed, b"bytes").expect("unseal failed");

    assert_eq!(payload.as_slice(), unsealed.as_slice());
}

#[test]
fn seal_unseal_tagged_roundtrip() {
    let vault = setup_vault();
    let profile = Profile { username: "ada".to_owned(), enabled: true };

    let sealed = vault.seal::<Local, _>(&profile).expect("seal failed");
    let unsealed: Profile = vault.unseal_local(&sealed).expect("unseal failed");

    assert_eq!(profile, unsealed);
}

#[test]
fn seal_bytes_respects_default_compression() {
    let vault = setup_vault();
    let payload = vec![0u8; 256];

    let sealed = vault.seal_bytes::<Local>(&payload, b"bytes").expect("seal failed");

    assert!(sealed.is_compressed(), "Expected payload to be compressed by default");
}

#[test]
fn unseal_bytes_requires_matching_context() {
    let vault = setup_vault();
    let payload = b"context-sensitive payload";

    let sealed = vault.seal_bytes::<Local>(payload, b"").expect("seal failed");
    let unsealed = vault.unseal_bytes::<Local>(&sealed, b"").expect("unseal failed");
    assert_eq!(payload.as_slice(), unsealed.as_slice());

    let wrong = vault.unseal_bytes::<Local>(&sealed, b"ctx");
    assert!(wrong.is_err(), "Unsealing should fail with non-empty context");
}
