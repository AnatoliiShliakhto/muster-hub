#![allow(unreachable_pub)]
#![allow(clippy::needless_pass_by_value)]

//! # Macros
//!
//! Procedural macros for the infrastructure.
//! This crate provides attribute macros to simplify boilerplate associated with
//! infrastructure components like the specialized async runtime.
//!
//! ## Usage
//! Add the crate under `dev-dependencies` for proc-macro consumers inside the workspace:
//! ```toml
//! [dev-dependencies]
//! mhub-derive = { path = "../infra/derive" }
//! ```
//!
//! See each macro’s docstring for examples; they are `ignore`d to avoid compiling in this crate,
//! but should be copied into consuming crates’ tests/examples as needed.

mod macros;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, ItemStruct, parse_macro_input};

/// Attribute macro to bootstrap the specialized Tokio runtime.
///
/// This macro transforms an `async fn main` into a standard `fn main` that initializes
/// a pre-configured Tokio runtime based on the specified performance profile.
///
/// # Arguments
///
/// * `high_performance` - Optimized for high-throughput server environments.
/// * `memory_efficient` - Optimized for low-footprint client or edge environments.
/// * `default` - Uses the default configuration (worker threads auto-detected based on available parallelism).
///
/// # Examples
///
/// ```rust,ignore
/// #[mhub_runtime::main(high_performance)]
/// async fn main() -> Result<(), ()> {
/// # Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    macros::runtime::expand_main(args.into(), input).into()
}

/// Professional attribute macro to define a standard API data model.
///
/// This macro ensures consistency across all DTOs (Data Transfer Objects) in the
/// platform by injecting common behaviors and constraints.
///
/// # Injected Behaviors
///
/// * **Derives**: Automatically adds `Debug`, `Serialize`, and `Deserialize` if missing.
/// * **`OpenAPI`**: Conditionally adds `utoipa::ToSchema` when the `server` feature is enabled.
/// * **Serde Policy**:
///     * `rename_all = "camelCase"` by default (can be overridden).
///     * `deny_unknown_fields` by default (can be disabled).
///
/// # Arguments
///
/// * `rename_all = "camelCase"` - Overrides the default Serde rename policy.
/// * `deny_unknown_fields = false` - Disables strict field checking.
///
/// # Example
///
/// ```rust,ignore
/// use mhub_derive::api_model;
///
/// #[api_model(rename_all = "snake_case", deny_unknown_fields = false)]
/// pub struct UserProfile {
///     pub id: String,
///     pub display_name: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn api_model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    macros::api::expand_api_model(attr.into(), input).into()
}

/// Attribute macro to bridge Axum handlers with `OpenAPI` documentation.
///
/// This macro wraps a standard async function and integrates it with `utoipa`.
///
/// # Arguments
///
/// Accepts standard `utoipa::path` arguments such as `get`, `post`, `path = "..."`,
/// `responses(...)`, and `tag = "..."`.
///
/// # Features
///
/// * **Documentation**: Registers handler metadata via `utoipa::path` when the `server` feature is enabled.
/// * **Linting**: Applies `#[allow(clippy::unused_async)]` to the handler to satisfy boilerplate
///   requirements of certain Axum extractors.
///
/// # Example
///
/// ```rust,ignore
/// use mhub_derive::api_handler;
///
/// #[api_handler(
///     get,
///     path = "/health",
///     responses((status = OK, body = HealthResponse)),
///     tag = "System"
/// )]
/// pub async fn health_handler() -> Result<(), ()> {
///     // ...
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn api_handler(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    macros::api::expand_api_handler(args.into(), input).into()
}

/// Attribute macro to generate vault-aware Serde behavior for structs.
///
/// This macro generates `Serialize` and `Deserialize` impls that preserve field
/// behavior and serde attributes. It also implements `mhub_vault::Tagged` using
/// the optional `tag = "..."` argument or the struct name, marks the type as
/// `mhub_vault::VaultSerde`, and provides `Debug`, `PartialEq`, `Eq`, and `Hash`.
///
/// # Results
/// Expands to `Serialize`/`Deserialize` impls for the annotated struct.
///
/// # Errors
/// Emits a compile-time error if the macro is applied to a non-struct or
/// a struct without named fields.
///
/// # Example
/// ```rust,ignore
/// use mhub_vault::prelude::*;
///
/// #[vault_model(tag = "v1.user_record")]
/// struct UserRecord {
///     username: String,
///     ssn: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn vault_model(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    macros::vault::expand_derive(args.into(), input).into()
}

/// A high-level attribute macro for defining domain-specific error enums.
///
/// This macro reduces boilerplate by transforming a standard enum into a fully-featured
/// error type integrated with the `MusterHub` infrastructure.
///
/// # Features
///
/// * **Automatic Derives**: Injects `#[derive(Debug, thiserror::Error)]`.
/// * **Type Aliasing**: Automatically creates a `Result<T>` type alias for the enum.
/// * **Context Support**: Generates a companion `...Ext` trait that adds `.context()`
///   to any `Result` that can be converted into this error type.
/// * **Standard Conversions**: Implements `From<T>` for variants containing a `#[source]` field,
///   enabling the use of the `?` operator for upstream errors.
/// * **Internal Fallback**: Provides specialized `From<&str>` and `From<String>` implementations
///   if an `Internal` variant is present.
///
/// # Requirements
///
/// 1. The macro must be applied to an **enum**.
/// 2. Variants that support context must include a `context: Option<Cow<'static, str>>` field.
/// 3. Variants wrapping external errors must include a `source: T` field or a field marked
///    with `#[source]`/`#[from]` (compatible with `thiserror`).
/// 4. Tuple or unit variants are rejected to keep error wiring explicit and reliable.
///
/// # Generated Items
///
/// * `Result<T>` type alias in the same module.
/// * `<ErrorName>Ext` trait with `.context(...)` for both `Result<T, ErrorName>` and
///   `Result<T, SourceError>` when a source field exists.
/// * `From<SourceError>` impls for variants with a source field and a context field.
/// * `From<&'static str>` and `From<String>` when an `Internal` variant is present.
///
/// # Example
///
/// ```rust,ignore
/// use mhub_derive::mhub_error;
/// use std::borrow::Cow;
///
/// #[mhub_error]
/// pub enum DatabaseError {
///     #[error("IO error{}: {source}", format_context(.context))]
///     Io {
///         #[source]
///         source: surrealdb::Error,
///         context: Option<Cow<'static, str>>,
///     },
///
///     #[error("Internal fault{}: {message}", format_context(.context))]
///     Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
/// }
///
/// // Usage:
/// fn fetch_data() -> Result<String> {
///     db.execute("SELECT...")
///         .context("Executing user lookup")? // Adds context to the SurrealDB error
///         .try_into()
///         .map_err(|_| "Failed to parse".into()) // Uses From<&str> for Internal variant
/// }
/// ```
#[proc_macro_attribute]
pub fn mhub_error(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    macros::error::expand_derive(input).into()
}

/// Attribute macro to define a Vertical Slice handle.
///
/// This macro transforms a struct into a full Slice pattern:
/// 1. Generates a thread-safe `Arc` wrapper.
/// 2. Implements `Deref` for transparent access to the inner state.
/// 3. Implements `FeatureSlice` for registration in the Kernel.
///
/// # Example
/// ```rust,ignore
/// #[mhub_derive::mhub_slice]
/// pub struct FeatureSlice {
///     pub name: String,
/// }
///
/// fn init() -> FeatureSlice {
///     let inner = FeatureSliceInner { name: "FeatureSlice".to_owned() };
///     FeatureSlice::new(inner)
/// }
/// ```
#[proc_macro_attribute]
pub fn mhub_slice(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);
    macros::slice::expand_slice(input).into()
}
