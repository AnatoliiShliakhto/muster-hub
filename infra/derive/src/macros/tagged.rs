use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr};

/// Expands the `#[derive(Tagged)]` macro.
pub fn expand_derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // 1. Determine the tag value (default to struct name)
    let mut tag_value = name.to_string();

    // 2. Parse #[tagged("...")] attribute
    for attr in &input.attrs {
        if attr.path().is_ident("tagged") {
            match attr.parse_args::<LitStr>() {
                Ok(lit) => tag_value = lit.value(),
                Err(err) => return err.to_compile_error(),
            }
        }
    }

    // 3. Generate the implementation
    quote! {
        #[automatically_derived]
        impl #impl_generics ::mhub_vault::types::Tagged for #name #ty_generics #where_clause {
            const TAG: &'static str = #tag_value;
        }
    }
}
