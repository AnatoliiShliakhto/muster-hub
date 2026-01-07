use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, ReturnType};

/// Expands the `#[mhub_runtime::main]` attribute macro.
#[must_use]
pub(crate) fn expand_main(args: TokenStream, input: ItemFn) -> TokenStream {
    if input.sig.asyncness.is_none() {
        return Error::new_spanned(
            &input.sig.ident,
            "the #[mhub_runtime::main] attribute can only be used on async functions",
        )
        .to_compile_error();
    }

    let name = &input.sig.ident;
    let body = &input.block;
    let vis = &input.vis;
    let attrs = &input.attrs;

    let ret_type = match &input.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let runtime_call = match parse_profile(args) {
        Ok(profile) => profile,
        Err(err) => return err,
    };

    quote! {
        use mhub_error::ErrorMetadata;

        #(#attrs)*
        #vis fn #name() {
            // --- PANIC HOOK START ---
            ::std::panic::set_hook(::std::boxed::Box::new(|info| {
                let location = info.location()
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "unknown location".to_string());

                let message = info.payload()
                    .downcast_ref::<&str>()
                    .map(|s| *s)
                    .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
                    .unwrap_or("no message provided");

                ::tracing::error!(
                    target: "panic",
                    location = %location,
                    "APPLICATION PANIC: {}",
                    message
                );

                ::std::eprintln!("\n=== APPLICATION PANIC ===\nLocation: {}\nMessage: {}\n", location, message);
            }));
            // --- PANIC HOOK END ---

            let config = #runtime_call;

            let rt = match ::mhub_runtime::build_runtime_with_config(&config) {
                Ok(runtime) => runtime,
                Err(e) => {
                    eprintln!("Critical Failure:\n\n{}\n", e.detailed());
                    ::std::process::exit(1);
                }
            };

            let result: #ret_type = rt.block_on(async move {
                #body
            });

            if let ::std::result::Result::Err(e) = result {
                let target = e.target();
                let kind = e.kind();
                let code = e.code();
                let context = e.context().as_ref().map(|c| c.to_string()).unwrap_or_default();
                let backtrace = e.backtrace().as_ref().map(|b| b.to_string()).unwrap_or_default();
                let details = e.to_json_string().unwrap_or_default();

                ::tracing::error!(
                    %target,
                    %code,
                    %kind,
                    %context,
                    %backtrace,
                    %details,
                    "Critical Failure\n\n{}",
                    e.detailed(),
                );

                //::std::eprintln!("\n=== CRITICAL FAILURE ===\n\n{}\n\n", e.detailed());

                ::std::process::exit(1);
            }
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
            "unknown runtime profile. Use: high_performance, memory_efficient, or default",
        )
        .to_compile_error()),
    }
}
