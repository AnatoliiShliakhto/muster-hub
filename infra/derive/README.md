# mhub-derive ✨

Procedural macros for MusterHub infra and features: runtime bootstrap, API models/handlers, vault
models, and rich error enums/slices.

## Macros

- `#[mhub_runtime::main(<profile>)]`: wrap `async fn main` with a configured Tokio runtime.
- `#[api_model]`: injects serde derives, camelCase JSON, `deny_unknown_fields`, and
  `utoipa::ToSchema` (when `server` feature is on in consumer). Supports
  `rename_all = "..."` and `deny_unknown_fields = false`.
- `#[api_handler(...)]`: bridges Axum handlers with `utoipa::path` metadata; applies
  `allow(clippy::unused_async)` and only emits OpenAPI metadata when `server` is enabled.
- `#[vault_model]`: generates Serde impls, implements `Tagged` using the optional
  `tag = "..."` argument or struct name, marks the type as `mhub_vault::VaultSerde` for vault APIs,
  and implements `Debug`, `PartialEq`, `Eq`, and `Hash`.
- `#[mhub_error]`: generates `thiserror` enums with `Result<T>` alias, context extension, and `From`
  for sources/internal.
- `#[mhub_slice]`: transforms a struct into a FeatureSlice (Arc/Deref) for kernel registration.

## Usage

Add to your crate (a workspace path shown):

```toml
[dependencies]
mhub-derive = { path = "../infra/derive" }
```

## Examples

```rust
#[api_model(rename_all = "snake_case", deny_unknown_fields = false)]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
}

use mhub_derive::vault_model;

#[vault_model(tag = "v1.user_record")]
struct UserRecord {
    username: String,
    ssn: String,
}

use mhub_derive::api_handler;

#[api_handler(
    get,
    path = "/health",
    responses((status = OK, description = "OK"))
)]
pub async fn health_handler() -> Result<(), ()> {
    Ok(())
}

#[mhub_error]
pub enum MyError {
    #[error("IO error: {source}, context: {}", format_context(.context))]
    Io {
        source: std::io::Error,
        context: Option<std::borrow::Cow<'static, str>>,
    },

    #[error("Internal fault{}: {message}", format_context(.context))]
    Internal {
        message: Cow<'static, str>,
        context: Option<Cow<'static, str>>,
    },
}
```

## Testing

- Macro sanity tests cover `vault_model`, `api_model` (serde camelCase), and `mhub_error` context
  wiring.
- For compile-time behavior (e.g., `api_handler`, `main`), consider adding `trybuild` tests in
  consumers.

## mhub_error Notes

- Context fields must be named `context` and typed as `Option<Cow<'static, str>>`.
- Source fields can be named `source` or marked with `#[source]`/`#[from]`.
- Tuple or unit variants are rejected to keep error wiring explicit.
- `#[cfg(...)]` attributes on variants are preserved on generated impls.
- The macro generates a `Result<T>` alias and an `<ErrorName>Ext` trait for `.context(...)`.

## mhub_error Examples

```rust
use mhub_derive::mhub_error;
use std::borrow::Cow;

#[mhub_error]
pub enum DemoError {
    #[error("IO error{}: {source}", format_context(.context))]
    Io {
        #[source]
        source: std::io::Error,
        context: Option<Cow<'static, str>>,
    },

    #[error("Internal error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
```
