# mhub-derive✨

Procedural macros for the MusterHub infrastructure.

This crate bundles the attribute macros used across the workspace to remove boilerplate for
Tokio runtimes, API and DTO definitions, vault serialization, vertical slices, and rich error
handling.

## Overview

- **Language:** Rust
- **Scope:** Developer-facing proc-macros consumed by other crates in the workspace
- **Key Benefits:** Consistent code generation, fewer handwritten impls, standardized error stories

## Macro Catalog

### Runtime & Infrastructure

#### `#[main(<profile>)]`

Transforms an `async fn main` into a standard `fn main` that boots the pre-configured Tokio runtime.
Supported profiles: `high_performance`, `memory_efficient`, `default`.

#### `#[mhub_slice]`

Builds the Vertical Slice boilerplate: wraps the struct in an `Arc`, implements `Deref`, and
registers it as a `FeatureSlice` in the Kernel.

### API & Web

#### `#[api_model(...)]`

Standardizes DTOs: injects serde derives, default `camelCase` renaming, `deny_unknown_fields`, and
`utoipa::ToSchema` when the `server` feature is enabled.

#### `#[api_handler(...)]`

Bridges Axum handlers with `utoipa::path` metadata, adds `#[allow(clippy::unused_async)]`, and
handles conditional docs compilation.

### Vault & Persistence

#### `#[vault_model(tag = "...")]`

Generates vault-aware `Serialize`/`Deserialize`, implements `mhub_vault::Tagged` (using an optional
`tag = "..."`) plus `mhub_vault::VaultSerde`, and derives `Debug`, `Eq`, and `Hash`.

### Error Handling

#### `#[mhub_error]`

The “senior” error macro. Converts an enum into a full-featured error type that integrates with
`ErrorCode`, autogenerates `Result<T>` aliases, `<ErrorName>Data` internals, and a `<ErrorName>Ext`
extension trait (context/code/kind helpers). Adds `From<Source>` impls, snake_case constructors, and
DetailedReport rendering.

## Usage Examples

### API & Vault Models

```rust,ignore
#[mhub_derive::api_model(rename_all = "snake_case")] 
pub struct UserProfile { 
    pub id: String, 
    pub display_name: String, 
}

#[mhub_derive::vault_model(tag = "v1.user_record")] 
struct UserRecord { 
    username: String, 
    ssn: String, 
}
``` 

### Advanced Error Handling

```rust,ignore 
#[mhub_derive::mhub_error] 
pub enum DatabaseError { 
    /// Will have ErrorCode::NotFound (404) 
    #[error(message = "User not found", code = 404)] 
    UserNotFound,

    /// Wraps external IO error with 500 status
    #[error(message = "Connection failed", source = std::io::Error, code = "InternalServerError")]
    ConnectionIssue,
}

fn connect() -> Result<(), DatabaseError> { 
    std::fs::read_to_string("config.db")
        .context("Missing database configuration")?; 
    Ok(()) 
}
```

## `#[mhub_error]` Attribute Reference

| Attribute | Description                      | Accepted Values                           |
|-----------|----------------------------------|-------------------------------------------|
| `message` | Human-readable error description | String literal                            |
| `code`    | Associated `ErrorCode`           | `u16`, `&str`, or `ErrorCode::Variant`    |
| `kind`    | Machine-readable slug/kind       | String (defaults to SHOUTY_SNAKE_CASE)    |
| `source`  | Upstream error to wrap           | Any type implementing `std::error::Error` |

**Generated API (for `MyError`):**

- `MyError::variant_name(Option<Cow<'static, str>>)` constructors
- `MyErrorExt` trait providing `.context()`, `.code()`, `.kind()` on `Result`
- `ErrorMetadata` impls for status code + backtrace retrieval
- `DetailedReport` helpers for inspection/testing `format!("{err}")`, `format!("{err:?}")`, `err.details()`

## Testing

Macro expansion tests live under `tests/`. Consumers should supplement them with integration tests
that exercise the generated APIs (e.g., `DetailedReport`, `ErrorMetadata`) to ensure trait bounds and
type safety remain intact.
