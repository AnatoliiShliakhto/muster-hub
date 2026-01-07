# mhub-domain 🧩

Pure domain types for MusterHub. Minimal dependencies (`serde`, `bitflags`) and no business logic or
I/O. Keep this crate lean to maximize reuse across server/client/tools.

## Contents

- Entity identifiers (`Entity`) with `as_str` and `TryFrom<&str>`.
- Constants for entity names.
- Configuration structs (server, database, storage, security).
- Feature flags/bitflags (see `features.rs`).

## Examples

```rust
use mhub_domain::entity::Entity;

assert_eq!(Entity::User.as_str(), "user");
assert_eq!(Entity::try_from("quiz").unwrap(), Entity::Quiz);
assert!(Entity::try_from("unknown").is_err());
```

## Config defaults

- `ServerConfig`: `0.0.0.0:4583`, no SSL by default.
- `DatabaseConfig`: `mem://`, namespace `mhub`, database `core`, default root creds.
- `StorageConfig`: `data_dir="."`, `static_dir="public"`.

## Guidance

- Do not add heavy deps; keep the domain pure.
- Extend types here when they are shared across layers; keep logic in higher layers.

## Tests

- Coverage for entity roundtrip, constants, config defaults/deserialization.
