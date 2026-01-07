use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemStruct;

pub fn expand_slice(input: ItemStruct) -> TokenStream {
    let inner_ident = &input.ident;
    let vis = &input.vis;

    // Generate the wrapper name: e.g., IdentityInner -> Identity
    let wrapper_name_str = inner_ident.to_string().replace("Inner", "");
    let wrapper_ident = format_ident!("{}", wrapper_name_str);

    quote! {
        #[derive(Debug)]
        #input

        #[derive(Debug, Clone)]
        #vis struct #wrapper_ident {
            inner: std::sync::Arc<#inner_ident>,
        }

        impl #wrapper_ident {
            pub fn new(inner: #inner_ident) -> Self {
                Self { inner: std::sync::Arc::new(inner) }
            }
        }

        impl std::ops::Deref for #wrapper_ident {
            type Target = #inner_ident;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl ::mhub_kernel::system::registry::FeatureSlice for #wrapper_ident {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    }
}
