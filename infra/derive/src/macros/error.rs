use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Ident, Type, Variant};

struct VariantMeta<'a> {
    ident: &'a Ident,
    source_ty: Option<&'a Type>,
    has_context: bool,
}

pub fn expand_derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let trait_name = format_ident!("{}Ext", name);

    let Data::Enum(data) = &input.data else {
        return quote! { compile_error!("mhub_error can only be derived for enums"); };
    };

    let variants: Vec<VariantMeta<'_>> = data.variants.iter().map(parse_variant).collect();

    let context_impl = generate_context_trait(name, &trait_name, &variants);
    let from_impls = variants.iter().filter_map(|v| generate_from_impl(name, &trait_name, v));
    let internal_impls = generate_internal_impls(name, &variants);

    quote! {
        #[derive(Debug, ::thiserror::Error)]
        #input

        pub type Result<T> = std::result::Result<T, #name>;
        #context_impl
        #(#from_impls)*
        #internal_impls

        #[allow(dead_code)]
        fn format_context(context: &Option<std::borrow::Cow<'static, str>>) -> std::borrow::Cow<'static, str> {
            context.as_ref().map_or(std::borrow::Cow::Borrowed(""), |c| std::borrow::Cow::Owned(format!(" ({c})")))
        }
    }
}

fn parse_variant(v: &Variant) -> VariantMeta<'_> {
    let mut source_ty = None;
    let mut has_context = false;

    if let Fields::Named(fields) = &v.fields {
        for field in &fields.named {
            let Some(f_ident) = &field.ident else { continue };
            if f_ident == "context" {
                has_context = true;
            }
            if f_ident == "source" {
                source_ty = Some(&field.ty);
            }
        }
    }

    VariantMeta { ident: &v.ident, source_ty, has_context }
}

fn generate_context_trait(
    name: &Ident,
    trait_name: &Ident,
    variants: &[VariantMeta<'_>],
) -> TokenStream {
    let context_variants = variants.iter().filter(|v| v.has_context).map(|v| v.ident);

    quote! {
        pub trait #trait_name<T> {
            fn with_context(self, context: impl Into<std::borrow::Cow<'static, str>>) -> Result<T>;
        }

        #[automatically_derived]
        impl<T> #trait_name<T> for Result<T> {
            #[inline]
            fn with_context(self, context: impl Into<std::borrow::Cow<'static, str>>) -> Self {
                self.map_err(|mut e| {
                    match &mut e {
                        #( #name::#context_variants { context: c, .. } => *c = Some(context.into()), )*
                        _ => {}
                    }
                    e
                })
            }
        }
    }
}

fn generate_from_impl(
    name: &Ident,
    trait_name: &Ident,
    v: &VariantMeta<'_>,
) -> Option<TokenStream> {
    if v.ident == "Internal" {
        return None;
    }
    let source_ty = v.source_ty?;
    let v_ident = v.ident;

    Some(quote! {
        #[automatically_derived]
        impl From<#source_ty> for #name {
            #[inline]
            fn from(source: #source_ty) -> Self { Self::#v_ident { source, context: None } }
        }

        impl<T> #trait_name<T> for std::result::Result<T, #source_ty> {
            #[inline]
            fn with_context(self, context: impl Into<std::borrow::Cow<'static, str>>) -> std::result::Result<T, #name> {
                self.map_err(|source| #name::#v_ident { source, context: Some(context.into()) })
            }
        }
    })
}

fn generate_internal_impls(name: &Ident, variants: &[VariantMeta<'_>]) -> TokenStream {
    if !variants.iter().any(|v| v.ident == "Internal") {
        return quote!();
    }

    quote! {
        impl From<&'static str> for #name {
            #[inline]
            fn from(s: &'static str) -> Self { Self::Internal { message: std::borrow::Cow::Borrowed(s), context: None } }
        }
        impl From<String> for #name {
            #[inline]
            fn from(s: String) -> Self { Self::Internal { message: std::borrow::Cow::Owned(s), context: None } }
        }
    }
}
