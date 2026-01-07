use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, ReturnType, Type};

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

    if !returns_result(&input.sig.output) {
        return Error::new_spanned(
            &input.sig.output,
            "The #[mhub_runtime::main] attribute requires a Result return type",
        )
        .to_compile_error();
    }

    let name = &input.sig.ident;
    let body = &input.block;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let output = &input.sig.output;

    // 2. Parse arguments to determine the RuntimeConfig preset
    let runtime_call = match parse_profile(args) {
        Ok(profile) => profile,
        Err(err) => return err,
    };

    // 3. Generate the wrapper function
    quote! {
        #(#attrs)*
        #vis fn #name() #output {
            let config = #runtime_call;
            let rt = ::mhub_runtime::build_runtime_with_config(&config)?;
            rt.block_on(async { #body })
        }
    }
}

fn parse_profile(args: TokenStream) -> Result<TokenStream, TokenStream> {
    if args.is_empty() {
        return Ok(quote! { ::mhub_runtime::RuntimeConfig::default() });
    }

    let ident: syn::Ident = syn::parse2(args).map_err(|err| err.to_compile_error())?;
    match ident.to_string().as_str() {
        "high_performance" => Ok(quote! { ::mhub_runtime::RuntimeConfig::high_performance() }),
        "memory_efficient" => Ok(quote! { ::mhub_runtime::RuntimeConfig::memory_efficient() }),
        "default" => Ok(quote! { ::mhub_runtime::RuntimeConfig::default() }),
        _ => Err(Error::new_spanned(
            ident,
            "Unknown runtime profile. Use: high_performance, memory_efficient, or default",
        )
        .to_compile_error()),
    }
}

fn returns_result(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    let Type::Path(path) = &**ty else {
        return false;
    };
    path.path.segments.last().is_some_and(|seg| seg.ident == "Result")
}
