use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemStruct};

/// Expands the `#[api_model]` attribute macro.
///
/// Automatically adds common derives (`Serialize`, `Deserialize`, `ToSchema`) and
/// configures Serde for camelCase and strict field checking.
pub fn expand_api_model(input: ItemStruct) -> TokenStream {
    quote! {
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[cfg_attr(feature = "server", derive(::utoipa::ToSchema))]
        #[serde(rename_all = "camelCase")]
        #[serde(deny_unknown_fields)]
        #input
    }
}

/// Expands the `#[api_handler]` attribute macro.
///
/// Integrates with `utoipa::path` for `OpenAPI` documentation while maintaining
/// clean handler signatures.
pub fn expand_api_handler(args: TokenStream, input: ItemFn) -> TokenStream {
    let body = &input.block;
    let sig = &input.sig;
    let vis = &input.vis;
    let attrs = &input.attrs;

    quote! {
        #(#attrs)*
        #[allow(clippy::unused_async)]
        #[::utoipa::path(#args)]
        #vis #sig {
            #body
        }
    }
}
