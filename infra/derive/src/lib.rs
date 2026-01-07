//! # Macros
//!
//! Procedural macros for the infrastructure.
//! This crate provides attribute macros to simplify boilerplate associated with
//! infrastructure components like the specialized async runtime.

use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::{Error, ItemFn, ItemStruct, parse_macro_input};

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

    // Ensure the function is async
    if input.sig.asyncness.is_none() {
        return Error::new_spanned(
            &input.sig.ident,
            "The #[mhub_runtime::main] attribute can only be used on async functions",
        )
        .to_compile_error()
        .into();
    }

    let args_str = args.to_string();
    let name = &input.sig.ident;
    let body = &input.block;
    let vis = &input.vis;
    let attrs = &input.attrs;

    // Map arguments to RuntimeConfig presets
    let runtime_call = match args_str.as_str() {
        "high_performance" => {
            quote! { ::mhub_runtime::RuntimeConfig::high_performance() }
        },
        "memory_efficient" => {
            quote! { ::mhub_runtime::RuntimeConfig::memory_efficient() }
        },
        _ => quote! { ::mhub_runtime::RuntimeConfig::default() },
    };

    // Expand the code
    let result = quote! {
        #(#attrs)*
        #vis fn #name() -> ::mhub_runtime::Result<()> {
            let config = #runtime_call;
            let rt = ::mhub_runtime::build_runtime_with_config(&config)?;
            rt.block_on(async { #body })
        }
    };

    result.into()
}

#[proc_macro_attribute]
pub fn api_model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[cfg_attr(feature = "api", derive(::utoipa::ToSchema))]
        #[serde(rename_all = "camelCase")]
        #[serde(deny_unknown_fields)]
        #input
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn api_handler(args: TokenStream, item: TokenStream) -> TokenStream {
    // Convert proc_macro::TokenStream to proc_macro2::TokenStream
    let args2 = TokenStream2::from(args);
    let input = parse_macro_input!(item as ItemFn);

    let body = &input.block;
    let sig = &input.sig;
    let vis = &input.vis;
    let attrs = &input.attrs;

    quote! {
        #(#attrs)*
        #[allow(clippy::unused_async)]
        // Use the converted args2 here
        #[::utoipa::path(#args2)]
        #vis #sig {
            #body
        }
    }
    .into()
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
    let input = parse_macro_input!(item as syn::DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // 1. Determine the tag value
    let mut tag_value = name.to_string();

    // 2. Parse attributes with professional error handling
    for attr in &input.attrs {
        if attr.path().is_ident("tagged") {
            let result = attr.parse_args::<syn::LitStr>().map(|lit| {
                tag_value = lit.value();
            });

            if let Err(err) = result {
                return err.to_compile_error().into();
            }
        }
    }

    // 3. Generate the implementation using split_for_impl for generics support
    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::mhub_vault::types::Tagged for #name #ty_generics #where_clause {
            const TAG: &'static str = #tag_value;
        }
    };

    TokenStream::from(expanded)
}