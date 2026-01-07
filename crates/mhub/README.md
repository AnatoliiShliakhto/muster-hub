# mhub 🎛️

Workspace facade that composes shared crates and feature slices. Stays thin: re-exports primitives
and initializes enabled features.

## Features

- `server`: enables kernel server parts and identity/audit/organization/licensing server slices.
- `client`: enables kernel client parts and identity client slice.
- `mhub-licensing`: pulls in licensing slice (on server).

## Usage

```toml
[dependencies]
mhub = { path = "../crates/mhub", features = ["server"] }
```

Server init:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let events = mhub_event_bus::EventBus::new();
    let slices = mhub::init(&config, &database, &events)?;

    // register slices into kernel state as needed

    Ok(())
}
```

Feature registry:

```rust
fn main() {
    if mhub::features::is_enabled("identity") {
        // conditional wiring
    }

    Ok(())
}
```

## Notes

- `init` currently wires identity/audit (and licensing when enabled); extend it as new slices are added.
- Keep business logic in feature crates; `mhub` only orchestrates.
