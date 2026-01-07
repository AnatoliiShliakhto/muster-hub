# mhub-desktop 🖥️

Desktop entrypoint built with Dioxus (desktop target). Provides a small wrapper for window config
and launch.

## Usage

```rust
fn main() {}
```

## Window config

- Title and size configurable via builder.
- Injects a responsive viewport meta tag by default.

## Notes

- `main` binary is currently empty; wire it to call your `DesktopApp` builder.
- Client feature is enabled on `mhub` for shared client-side pieces.
