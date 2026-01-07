# mhub-kernel 🧠

Lightweight kernel utilities shared across server/client slices: config loading, security helpers,
slice registry, and safe ID generation.

## Modules

- `config` (non-wasm): layered config loader (file + `MHUB__` env overrides).
- `security::resource`: resource ID guard to prevent table spoofing.
- `system::registry`: type-erased feature slice registry.
- `server` (feature-gated): router/state glue for Axum-based services.
- `safe_nanoid!`: generates unambiguous NanoIDs (no confusing characters).

## Examples

```rust
// Safe ID
use mhub_kernel::safe_nanoid;

fn main() {
    let id = safe_nanoid!();
    assert_eq!(id.len(), 12);

    // Resource guard
    use mhub_kernel::security::resource::ResourceGuard;
    assert_eq!(ResourceGuard::verify("123", "user").unwrap(), "user:123");
}
```

## Config loader

- `MHUB__` env prefix; nested fields via `__` (e.g., `MHUB__DATABASE__URL`).
- Uses `config` crate to merge file + env; returns a deserialized config type.

## Tests

- Coverage for `safe_nanoid`, resource guard, and slice registry downcasts.

## Guidance

- Keep kernel lean; feature-specific logic belongs in slices/features.
- Prefer `FromRef` for Axum extraction of shared state.
