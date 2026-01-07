#![allow(unreachable_pub)]
#![allow(clippy::needless_pass_by_value)]

//! # Macros
//!
//! Procedural macros for the infrastructure.
//! This crate provides attribute macros to simplify boilerplate associated with
//! infrastructure components like the specialized async runtime.

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
/// ```rust
/// #[mhub_runtime::main(high_performance)]
/// async fn main() -> mhub_runtime::Result<()> {
///     mhub_server::run().await
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
/// * **Derives**: Automatically adds `Debug`, `Serialize`, and `Deserialize`.
/// * **`OpenAPI`**: Conditionally adds `utoipa::ToSchema` when the `server` feature is enabled.
/// * **Serde Policy**:
///     * `rename_all = "camelCase"`: Ensures frontend-friendly JSON naming.
///     * `deny_unknown_fields`: Prevents accidental data corruption/bugs from malformed input.
///
/// # Example
///
/// ```rust
/// #[api_model]
/// pub struct UserProfile {
///     pub id: String,
///     pub display_name: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn api_model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    macros::api::expand_api_model(input).into()
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
/// * **Documentation**: Automatically registers the handler's metadata for the Swagger/Scalar UI.
/// * **Linting**: Applies `#[allow(clippy::unused_async)]` to the handler to satisfy boilerplate
///   requirements of certain Axum extractors.
///
/// # Example
///
/// ```rust
/// #[api_handler(
///     get,
///     path = "/health",
///     responses((status = OK, body = HealthResponse)),
///     tag = "System"
/// )]
/// pub async fn health_handler() -> impl IntoResponse {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn api_handler(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    macros::api::expand_api_handler(args.into(), input).into()
}

/// Procedural macro to derive the `Tagged` trait.
///
/// This macro automatically implements `::mhub_vault::types::Tagged` for a struct.
/// By default, it uses the struct's name as the tag. You can override this using
/// the `#[tagged("custom_tag")]` attribute.
///
/// # Example
/// ```rust
/// #[derive(Tagged)]
/// #[tagged("v1.user_profile")]
/// struct UserProfile { ... }
/// ```
#[proc_macro_derive(Tagged, attributes(tagged))]
pub fn derive_tagged(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    macros::tagged::expand_derive(input).into()
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
/// * **Context Support**: Generates a companion `...Ext` trait that adds `.with_context()`
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
/// 3. Variants wrapping external errors must include a `source: T` field (compatible with `thiserror`).
///
/// # Example
///
/// ```rust
/// #[mhub_error]
/// pub enum DatabaseError {
///     #[error("Query failed{}: {source}", format_context(.context))]
///     Query {
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
/// fn fetch_data() -> DatabaseResult<String> {
///     db.execute("SELECT...")
///         .with_context("Executing user lookup")? // Adds context to the SurrealDB error
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
/// This macro transforms an `Inner` struct into a full Slice pattern:
/// 1. Generates a thread-safe `Arc` wrapper.
/// 2. Implements `Deref` for transparent access to the inner state.
/// 3. Implements `FeatureSlice` for registration in the Kernel.
#[proc_macro_attribute]
pub fn mhub_slice(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemStruct);
    macros::slice::expand_slice(input).into()
}