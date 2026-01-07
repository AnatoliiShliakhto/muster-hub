# mhub-error

A high-performance, structured error-handling framework tailored for Domain-Driven Design (DDD) and Vertical Slice
Architecture (VSA). This crate turns raw technical failures into meaningful, categorized, and traceable domain errors
that benefit both developers (tree reports) and machines (JSON logging).

## Key Features

- **Zero-Boilerplate Macros:** Use `#[error]` to generate full implementations: `From` conversions, context extensions,
  and more.
- **Semantic Categorization:** Every error maps to an `ErrorCode` (aligned with HTTP semantics) plus a machine-readable
  slug.
- **Hierarchical Reporting:** Built-in tree formatter for visualizing entire cause chains at a glance.
- **Observability Ready:** `json` feature enables structured logging for ELK, Loki, or Datadog pipelines.
- **Deep Interop:** Pre-built adapters for `std::io`, `serde_json`, `reqwest`, `surrealdb`, and more.

## Usage

### 1. Define Your Domain Errors

The `#[error]` macro (re-exported from `mhub-derive`) handles the boilerplate.

```rust 
use mhub_error::error;

#[error]
pub enum UserError {
    #[error(message = "User not found", code = 404, kind = "USER_NOT_FOUND")]
    NotFound,

    #[error(message = "Database connection failed", source = std::io::Error, code = 503)]
    StorageFailure,

    #[error(code = 400)]
    InvalidInput,
}
``` 

### 2. Wrapping and Context

Each generated `<ErrorName>Ext` trait provides `.with_context(...)` for layering high-level details.

```rust 
fn get_user_config() -> Result<String, UserError> {
    std::fs::read_to_string("config.json")
        .with_context("Missing required user profile configuration")?;
    Ok("config content".into())
}
```

## Inspection & Reporting

### Human-Readable Tree Reports

- `Display` yields a concise message.
- `.detailed()` renders the entire cause chain.

```rust,ignore
if let Err(err) = get_user_config() { 

    // × Missing required user profile configuration
    // │
    // ├─ Caused by:
    // │  1: Database connection failed 
    // │  2: No such file or directory (os error 2) 
    // │
    // ╰─ Help: Verify that config.json exists in the root directory
    
    eprintln!("{}", err.detailed()); 
}
``` 

### Structured JSON Logging

Enable the `json` feature to serialize errors for telemetry pipelines.

```rust,ignore 
let log_entry = err.to_serializable(); 
let serialized = serde_json::to_string(&log_entry)?;
```

**-OR-**

```rust,ignore 
let serialized = err.to_json_string()?;
```

## Interoperability

| Feature  | Crate        | Mapping Strategy                               | Default Error Code |
|----------|--------------|------------------------------------------------|--------------------|
| std      | std::io      | Maps NotFound, PermissionDenied, etc.          | 404, 403, 500      |
| json     | serde_json   | Distinguishes syntax vs. data mismatch         | 400 vs 422         |
| reqwest  | reqwest      | Handles timeouts and upstream status codes     | 504, 502, 500      |
| surreal  | surrealdb    | Maps constraint violations and missing records | 409, 404, 500      |
| webtoken | jsonwebtoken | Maps invalid signatures                        | 401, 403, 500      |
| tokio    | tokio        | Maps task cancellation                         | 499                |

## Core Architecture

### Metadata Accessors

Every error produced by the macro supports:

- `message()`: Returns the human-readable error description.
- `code()`: Returns the HTTP-aligned ErrorCode (e.g., `NotFound`).
- `kind()`: Returns a machine-readable identifier or slug (e.g., `USER_NOT_FOUND`).
- `target()`: Identifies the subsystem or crate where the error originated.
- `context()`: Returns additional dynamic context attached to the error.
- `help()`: Returns troubleshooting advice or suggested actions.
- `backtrace()`: Provides the captured stack trace, if available.
- `detailed()`: Returns a formatter for rendering a structured, hierarchical error report.
- `chain()`: Returns an iterator over the full causal chain of the error.
- `to_serializable()`: Maps the error and its metadata into a DTO for serialization.
- `to_json_string()`: Serializes the error report into a JSON string (requires `json` feature).

### **Fluent Builder API (`ErrorExt`)**

All extension methods support method-chaining and are implemented for both the error types and Result<T, E>:

- `with_message(...)`: Overrides the default error message.
- `with_code(...)`: Assigns a specific ErrorCode to the error.
- `with_kind(...)`: Overrides the machine-readable slug.
- `with_context(...)`: Attaches a high-level description of the operation that failed.
- `with_context_fn(...)`: Lazily attaches context using a closure (useful for expensive string formatting).
- `with_help(...)`: Adds troubleshooting instructions for the end-user or operator.
- `with_help_fn(...)`: Lazily adds troubleshooting instructions for the end-user or operator.

```rust,ignore 
let result = get_user_config()
    .with_context("Failed to retrieve user configuration")
    .with_help_fn(|| format!("For more details, refer to the documentation at {}", "https://example.com/docs"));
```

### `ErrorCode`

A strongly typed enum representing standard failure modes with `u16` conversions for Axum/HTTP.

## Contributing

This crate is foundational to the MusterHub stack. When adding new integrations, keep them feature-gated to minimize the
dependency footprint for lightweight builds (e.g., WASM/Dioxus).
