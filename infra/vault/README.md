# mhub-vault 🛡️

Secure, domain-aware AEAD vault with algorithm agility, HKDF key derivation, optional LZ4
compression, and type-level tagging for AAD binding.

## Features

- **Domains:** Separate keys for `Local` (node-bound) and `Fleet` (cluster-shared) data.
- **Ciphers:** Pluggable `VaultCipher` (defaults to AES-256-GCM; ChaCha20-Poly1305 available).
- **AAD binding:** Type-level `Tagged` for structured payloads and explicit byte contexts for raw
  payloads.
- **Compression:** Optional LZ4 block compression before encryption.
- **Memory hygiene:** HKDF keys zeroized on builder drop; key derivation via HKDF-SHA256.

## Quick start

```rust
use mhub_vault::prelude::*;

fn main() -> Result<(), VaultError> {
    let vault = Vault::<Aes>::builder()
        .derived_keys("master-secret", "salt", "machine-id")?
        .build()?;

    #[vault_model(tag = "v1.secret")]
    struct Secret {
        value: String,
    }

    let secret = Secret { value: "sensitive data".to_owned() };
    let sealed = secret.seal_local(&vault)?;
    let unsealed: Secret = vault.unseal_local(&sealed)?;
    assert_eq!(secret, unsealed);
    Ok(())
}
```

## Tagged payloads

`vault_model` implements `Tagged` automatically. Use `#[vault_model(tag = "...")]` to pin a stable
AAD label.

```rust
use mhub_vault::prelude::*;
use serde::{Deserialize, Serialize};

#[vault_model(tag = "v1.user_profile")]
#[derive(Clone)]
struct UserProfile {
    name: String
}

fn main() -> Result<(), VaultError> {
    let vault = Vault::<ChaCha>::builder()
        .derived_keys("master-secret", "salt", "machine-id")?
        .build()?;

    let user = UserProfile { name: "Ada".into() };
    let sealed = user.seal_local(&vault)?;
    let unsealed: UserProfile = vault.unseal_local(&sealed)?;
    assert_eq!(user, unsealed);
    Ok(())
}
```

## Domains & contexts

- Use `Local` for node-bound secrets; `Fleet` for cluster-shared data.
- For compact internal storage, use `seal::<Local, _>(&data)` and `unseal_local`.
- For raw bytes, use `seal_bytes::<Local>(&data, b\"ctx\")` and
  `unseal_local_bytes(payload, b\"ctx\")` (or `unseal_fleet_bytes`).

## Testing & benches

- Property tests cover round-trips across domains.
- Benchmarks (`cargo bench -p mhub-vault`) measure seal/unseal throughput.

## Safety notes

- Nonce strategy: random 96-bit per op; ensure high-quality RNG (defaults to `rand`).
- HKDF inputs: choose strong `ikm`, distinct `salt`, and stable `id` for `Local`.
- Consider pinning ciphers/versions in production; pre-release crypto crates are used.
