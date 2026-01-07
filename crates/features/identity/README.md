# mhub-identity 🔐

Identity & Access Management feature slice (placeholder). Wraps shared kernel slice wiring and is
ready to host identity domain models/services.

## Status

- Currently, initializes an `Identity` slice with no additional state. Extend as domain/services are
  added.
- Feature-gated: `server` / `client` propagate to `mhub-kernel` for Axum/Dioxus contexts.

## Usage

```rust
let slice = mhub_identity::init()?; // returns InitializedSlice for kernel registry
```

## Tests

- Init smoke test ensures slice registration succeeds.

## Next steps

- Add domain models (users, roles, permissions) under `src/domain`.
    - Define repositories in `domain::repository` and services under `services/`.
    - Add API handlers (server) and client bindings as needed.
- Wire persistence/event bus when available and extend `init` to construct state.
