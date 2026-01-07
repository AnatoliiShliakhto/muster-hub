mod generation;
mod parsing;
mod transform;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemEnum, parse_macro_input, parse_quote};

pub(crate) fn expand_derive(input: TokenStream) -> TokenStream {
    let mut item_enum = parse_macro_input!(input as ItemEnum);

    let variants_info = match parsing::parse_and_clean_variants(&mut item_enum) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    item_enum.attrs.push(parse_quote!(#[derive(Debug)]));

    transform::transform_enum_variants(&mut item_enum, &variants_info);

    let impl_blocks = generation::generate_implementations(&item_enum.ident, &variants_info);

    let expanded = quote! {
        pub use mhub_error::{ErrorMetadata, ErrorCode};
        #item_enum
        #impl_blocks
    };

    TokenStream::from(expanded)
}
