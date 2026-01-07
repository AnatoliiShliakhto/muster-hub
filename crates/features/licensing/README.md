# mhub-licensing 🔑

Cryptographic licensing engine using Ed25519 signatures. Supports license issuance (gated by
`issuance` feature) and lightweight validation for production binaries.

## Architecture

- **Validator:** Always available; checks expiry, hardware constraint, and Ed25519 signature.
- **Generator (`issuance` feature):** Vendor-side keygen and license signing (used by `xtask`).
- **Machine binding:** Optional fuzzy hardware matching via `machineid-rs`.
- **Features:** Bitflags from `mhub-domain` to control enabled capabilities.

## Data model

```rust
pub struct LicenseData {
    customer: String,
    customer_alias: Option<String>,
    constraint: MachineConstraint, // Any | Threshold { ids, min_matches }
    features: FeatureSet,
    salt: Vec<u8>,
    issued: i64,
    expires: i64,
}

pub struct SignedLicense {
    data: LicenseData,
    signature: Vec<u8>, // base64 in JSON
}
```

## Validation

```rust
use mhub_licensing::{SignedLicense, LicenseError};
use mhub_licensing::validator::validate_license;

fn main() -> Result<(), anyhow::Error> {
    let license = SignedLicense::from_json(&json_str)?;
    let pubkey: [u8; 32] = vec![]; // vendor public key
    validate_license(&license, &pubkey)?;

    Ok(())
}
```

## Issuance (vendor side)

Enable `issuance` to sign licenses (e.g., via `cargo xtask license`):

```toml
[features]
issuance = ["rand"]
```

## Formats

- JSON (human-readable; signature/salt base64-encoded).
- Postcard (compact binary) via `encode_bin` / `decode_bin`.

## Testing

- Integration tests cover JSON/bin roundtrip, signature validation, and expiry rejection.

## Safety notes

- Protect the signing key; distribute only the public key with products.
- Ensure system clock sanity; validation rejects clocks before issuance or after expiry.
