use fxhash::FxHashSet;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Type, Variant};

struct VariantMeta<'a> {
    ident: &'a Ident,
    source_ty: Option<&'a Type>,
    source_field: Option<&'a Ident>,
    has_context: bool,
    cfg_attrs: Vec<Attribute>,
}

pub fn expand_derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let trait_name = format_ident!("{}Ext", name);

    let Data::Enum(data) = &input.data else {
        return quote! { compile_error!("mhub_error can only be derived for enums"); };
    };

    let variants: Vec<VariantMeta<'_>> = match data.variants.iter().map(parse_variant).collect() {
        Ok(v) => v,
        Err(err) => return err,
    };
    if let Some(err) = variants_error(&variants) {
        return err;
    }

    let derived_traits = derived_trait_names(&input);
    let mut derive_tokens = Vec::new();
    if !derived_traits.contains("Debug") {
        derive_tokens.push(quote! { Debug });
    }
    if !derived_traits.contains("Error") {
        derive_tokens.push(quote! { ::thiserror::Error });
    }
    let extra_derives = if derive_tokens.is_empty() {
        quote! {}
    } else {
        quote! { #[derive(#(#derive_tokens),*)] }
    };

    let context_impl = generate_context_trait(name, &trait_name, &variants);
    let from_impls = variants.iter().filter_map(|v| generate_from_impl(name, &trait_name, v));
    let internal_impls = generate_internal_impls(name, &variants);

    quote! {
        #[allow(non_shorthand_field_patterns)]
        #extra_derives
        #input

        #context_impl
        #(#from_impls)*
        #internal_impls

        #[allow(dead_code)]
        fn format_context(context: &Option<std::borrow::Cow<'static, str>>) -> std::borrow::Cow<'static, str> {
            context.as_ref().map_or(std::borrow::Cow::Borrowed(""), |c| std::borrow::Cow::Owned(format!(" ({c})")))
        }
    }
}

fn parse_variant(v: &Variant) -> Result<VariantMeta<'_>, TokenStream> {
    let Fields::Named(fields) = &v.fields else {
        return Err(syn::Error::new_spanned(
            v,
            "mhub_error requires named fields for source/context handling",
        )
        .to_compile_error());
    };

    let context_field = find_context_field(fields)?;
    let source_field = find_source_field(fields);
    let cfg_attrs = v.attrs.iter().filter(|attr| attr.path().is_ident("cfg")).cloned().collect();

    Ok(VariantMeta {
        ident: &v.ident,
        source_ty: source_field.map(|field| &field.ty),
        source_field: source_field.and_then(|field| field.ident.as_ref()),
        has_context: context_field.is_some(),
        cfg_attrs,
    })
}

fn find_context_field(fields: &syn::FieldsNamed) -> Result<Option<&syn::Field>, TokenStream> {
    for field in &fields.named {
        let Some(ident) = &field.ident else { continue };
        if ident != "context" {
            continue;
        }
        if !is_context_type(&field.ty) {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "context field must be Option<Cow<'static, str>>",
            )
            .to_compile_error());
        }
        return Ok(Some(field));
    }

    Ok(None)
}

fn find_source_field(fields: &syn::FieldsNamed) -> Option<&syn::Field> {
    fields.named.iter().find(|field| {
        let is_source_name = field.ident.as_ref().is_some_and(|ident| ident == "source");
        is_source_name || field_has_attr(field, "source") || field_has_attr(field, "from")
    })
}

fn generate_context_trait(
    name: &Ident,
    trait_name: &Ident,
    variants: &[VariantMeta<'_>],
) -> TokenStream {
    let context_variants = variants.iter().filter(|v| v.has_context).map(|v| {
        let cfg_attrs = &v.cfg_attrs;
        let ident = v.ident;
        quote! { #(#cfg_attrs)* #name::#ident { context: c, .. } => *c = Some(context.into()), }
    });

    quote! {
        pub trait #trait_name<T> {
            fn context(self, context: impl Into<std::borrow::Cow<'static, str>>) -> Result<T, #name>;
        }

        #[automatically_derived]
        impl<T> #trait_name<T> for Result<T, #name> {
            #[inline]
            fn context(self, context: impl Into<std::borrow::Cow<'static, str>>) -> Self {
                self.map_err(|mut e| {
                    match &mut e {
                        #( #context_variants )*
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
    let source_field = v.source_field?;
    let v_ident = v.ident;
    let cfg_attrs = &v.cfg_attrs;

    Some(quote! {
        #(#cfg_attrs)*
        #[automatically_derived]
        impl From<#source_ty> for #name {
            #[inline]
            fn from(#source_field: #source_ty) -> Self { Self::#v_ident { #source_field, context: None } }
        }

        #(#cfg_attrs)*
        impl<T> #trait_name<T> for std::result::Result<T, #source_ty> {
            #[inline]
            fn context(self, context: impl Into<std::borrow::Cow<'static, str>>) -> std::result::Result<T, #name> {
                self.map_err(|#source_field| #name::#v_ident { #source_field, context: Some(context.into()) })
            }
        }
    })
}

fn generate_internal_impls(name: &Ident, variants: &[VariantMeta<'_>]) -> TokenStream {
    let internal = variants.iter().find(|v| v.ident == "Internal");
    let Some(internal) = internal else {
        return quote!();
    };
    let cfg_attrs = &internal.cfg_attrs;

    quote! {
        #(#cfg_attrs)*
        impl From<&'static str> for #name {
            #[inline]
            fn from(s: &'static str) -> Self { Self::Internal { message: std::borrow::Cow::Borrowed(s), context: None } }
        }
        #(#cfg_attrs)*
        impl From<String> for #name {
            #[inline]
            fn from(s: String) -> Self { Self::Internal { message: std::borrow::Cow::Owned(s), context: None } }
        }
    }
}

fn field_has_attr(field: &syn::Field, name: &str) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn derived_trait_names(input: &DeriveInput) -> FxHashSet<String> {
    let mut traits = FxHashSet::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                traits.insert(ident.to_string());
            } else if let Some(ident) = meta.path.segments.last().map(|seg| seg.ident.to_string()) {
                traits.insert(ident);
            }
            Ok(())
        });
    }

    traits
}

fn variants_error(variants: &[VariantMeta<'_>]) -> Option<TokenStream> {
    for v in variants {
        if v.source_ty.is_some() && !v.has_context {
            return Some(
                syn::Error::new_spanned(
                    v.ident,
                    "mhub_error requires `context: Option<Cow<'static, str>>` for variants with a source",
                )
                .to_compile_error(),
            );
        }
    }
    None
}

fn is_context_type(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(Type::Path(inner_path))) = args.args.first() else {
        return false;
    };
    let Some(inner_seg) = inner_path.path.segments.last() else {
        return false;
    };
    if inner_seg.ident != "Cow" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(inner_args) = &inner_seg.arguments else {
        return false;
    };
    let mut args_iter = inner_args.args.iter();
    let Some(syn::GenericArgument::Lifetime(lt)) = args_iter.next() else {
        return false;
    };
    if lt.ident != "static" {
        return false;
    }
    let Some(syn::GenericArgument::Type(Type::Path(str_path))) = args_iter.next() else {
        return false;
    };
    let Some(str_seg) = str_path.path.segments.last() else {
        return false;
    };
    str_seg.ident == "str"
}
