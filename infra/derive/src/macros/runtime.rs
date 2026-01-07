use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemFn};

/// Expands the `#[mhub_runtime::main]` attribute macro.
#[must_use]
pub fn expand_main(args: TokenStream, input: ItemFn) -> TokenStream {
    // 1. Validation: Ensure the function is async
    if input.sig.asyncness.is_none() {
        return Error::new_spanned(
            &input.sig.ident,
            "The #[mhub_runtime::main] attribute can only be used on async functions",
        )
        .to_compile_error();
    }

    let name = &input.sig.ident;
    let body = &input.block;
    let vis = &input.vis;
    let attrs = &input.attrs;

    // 2. Parse arguments to determine the RuntimeConfig preset
    let args_str = args.to_string();
    let runtime_call = match args_str.trim() {
        "high_performance" => {
            quote! { ::mhub_runtime::RuntimeConfig::high_performance() }
        },
        "memory_efficient" => {
            quote! { ::mhub_runtime::RuntimeConfig::memory_efficient() }
        },
        _ => quote! { ::mhub_runtime::RuntimeConfig::default() },
    };

    // 3. Generate the wrapper function
    quote! {
        #(#attrs)*
        #vis fn #name() -> ::mhub_runtime::Result<()> {
            let config = #runtime_call;
            let rt = ::mhub_runtime::build_runtime_with_config(&config)?;
            rt.block_on(async { #body })
        }
    }
}
