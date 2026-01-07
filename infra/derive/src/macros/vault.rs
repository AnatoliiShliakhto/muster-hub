use fxhash::FxHashSet;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{Data, DeriveInput, Fields, Ident, Lit, LitStr, Meta, Type, parse_quote};

struct FieldTokens {
    serialize_fields: Vec<TokenStream>,
    serialize_helper_fields: Vec<TokenStream>,
    deserialize_helper_fields: Vec<TokenStream>,
    field_idents: Vec<Ident>,
    field_types: Vec<Type>,
}

fn parse_tag_literal(args: TokenStream, input: &DeriveInput) -> Result<LitStr, TokenStream> {
    let name = &input.ident;
    let mut tag_literal: Option<LitStr> = None;

    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let metas = match parser.parse2(args) {
        Ok(metas) => metas,
        Err(err) => return Err(err.to_compile_error()),
    };

    for meta in metas {
        let Meta::NameValue(name_value) = meta else {
            return Err(
                syn::Error::new_spanned(meta, "Expected `tag = \"...\"`").to_compile_error()
            );
        };

        if !name_value.path.is_ident("tag") {
            return Err(syn::Error::new_spanned(
                name_value.path,
                "Only `tag = \"...\"` is supported",
            )
            .to_compile_error());
        }

        if tag_literal.is_some() {
            return Err(syn::Error::new_spanned(name_value, "Duplicate `tag = \"...\"` argument")
                .to_compile_error());
        }

        let lit = match &name_value.value {
            syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                Lit::Str(lit) => lit.clone(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        &name_value.value,
                        "Expected string literal for `tag = \"...\"`",
                    )
                    .to_compile_error());
                },
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    &name_value.value,
                    "Expected string literal for `tag = \"...\"`",
                )
                .to_compile_error());
            },
        };

        tag_literal = Some(lit);
    }

    Ok(tag_literal
        .unwrap_or_else(|| LitStr::new(&name.to_string(), proc_macro2::Span::call_site())))
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
            }
            Ok(())
        });
    }

    traits
}

fn named_fields(input: &DeriveInput) -> Result<Vec<syn::Field>, TokenStream> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields.named.iter().cloned().collect()),
            Fields::Unnamed(_) | Fields::Unit => Err(syn::Error::new_spanned(
                &input.ident,
                "Vault derive only supports structs with named fields",
            )
            .to_compile_error()),
        },
        _ => Err(syn::Error::new_spanned(&input.ident, "Vault derive only supports structs")
            .to_compile_error()),
    }
}

fn build_field_tokens(
    fields: Vec<syn::Field>,
    helper_mod: &Ident,
) -> Result<FieldTokens, TokenStream> {
    let mut serialize_fields = Vec::new();
    let mut serialize_helper_fields = Vec::new();
    let mut deserialize_helper_fields = Vec::new();
    let mut field_idents = Vec::new();
    let mut field_types = Vec::new();

    for field in fields {
        let attrs = field.attrs;
        let attrs_for_serialize = attrs.clone();
        let attrs_for_deserialize = attrs;
        let Some(ident) = field.ident else {
            return Err(syn::Error::new_spanned(
                helper_mod,
                "Vault derive only supports named fields",
            )
            .to_compile_error());
        };
        let ty = &field.ty;

        serialize_fields.push(quote! {
            #ident: &self.#ident,
        });
        serialize_helper_fields.push(quote! {
            #(#attrs_for_serialize)*
            pub(super) #ident: &'__mhub_vault_serde #ty
        });
        deserialize_helper_fields.push(quote! {
            #(#attrs_for_deserialize)*
            pub(super) #ident: #ty
        });

        field_idents.push(ident);
        field_types.push(ty.clone());
    }

    Ok(FieldTokens {
        serialize_fields,
        serialize_helper_fields,
        deserialize_helper_fields,
        field_idents,
        field_types,
    })
}

fn helper_mod_tokens(
    helper_mod: &Ident,
    helper_impl_generics: &impl quote::ToTokens,
    helper_where_clause: Option<&syn::WhereClause>,
    impl_generics: &impl quote::ToTokens,
    where_clause: Option<&syn::WhereClause>,
    fields: &FieldTokens,
) -> TokenStream {
    let FieldTokens { serialize_helper_fields, deserialize_helper_fields, .. } = fields;

    quote! {
        #[allow(non_snake_case, non_camel_case_types, unused_imports)]
        mod #helper_mod {
            use super::*;

            #[derive(::mhub_vault::serde::Serialize)]
            pub struct SerializeHelper #helper_impl_generics #helper_where_clause {
                #(#serialize_helper_fields,)*
            }

            #[derive(::mhub_vault::serde::Deserialize)]
            pub struct DeserializeHelper #impl_generics #where_clause {
                #(#deserialize_helper_fields,)*
            }
        }
    }
}

fn serde_impl_tokens(
    name: &Ident,
    impl_generics: &impl quote::ToTokens,
    ty_generics: &impl quote::ToTokens,
    where_clause: Option<&syn::WhereClause>,
    helper_mod: &Ident,
    fields: &FieldTokens,
) -> TokenStream {
    let FieldTokens { serialize_fields, field_idents, .. } = fields;

    quote! {
        #[automatically_derived]
        impl #impl_generics ::mhub_vault::serde::Serialize for #name #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::mhub_vault::serde::Serializer,
            {
                let helper = #helper_mod::SerializeHelper {
                    #(#serialize_fields)*
                };
                ::mhub_vault::serde::Serialize::serialize(&helper, serializer)
            }
        }

        #[automatically_derived]
        impl<'de> ::mhub_vault::serde::Deserialize<'de> for #name #ty_generics #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::mhub_vault::serde::Deserializer<'de>,
            {
                let helper =
                    #helper_mod::DeserializeHelper #ty_generics ::deserialize(deserializer)?;
                Ok(Self {
                    #(#field_idents: helper.#field_idents,)*
                })
            }
        }
    }
}

fn where_clause_with_bounds(
    base: Option<&syn::WhereClause>,
    bounds: impl IntoIterator<Item = syn::WherePredicate>,
) -> TokenStream {
    let mut predicates = base.map(|w| w.predicates.clone()).unwrap_or_default();
    predicates.extend(bounds);

    if predicates.is_empty() {
        quote! {}
    } else {
        quote! { where #predicates }
    }
}

fn debug_impl_tokens(
    name: &Ident,
    impl_generics: &impl quote::ToTokens,
    ty_generics: &impl quote::ToTokens,
    where_clause: Option<&syn::WhereClause>,
    fields: &FieldTokens,
) -> TokenStream {
    let FieldTokens { field_idents, field_types, .. } = fields;
    let bounds = field_types.iter().map(|ty| syn::parse_quote!(#ty: ::core::fmt::Debug));
    let where_clause = where_clause_with_bounds(where_clause, bounds);

    quote! {
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut builder = f.debug_struct(stringify!(#name));
                #(builder.field(stringify!(#field_idents), &self.#field_idents);)*
                builder.finish()
            }
        }
    }
}

fn partial_eq_impl_tokens(
    name: &Ident,
    impl_generics: &impl quote::ToTokens,
    ty_generics: &impl quote::ToTokens,
    where_clause: Option<&syn::WhereClause>,
    fields: &FieldTokens,
) -> TokenStream {
    let FieldTokens { field_idents, field_types, .. } = fields;
    let bounds = field_types.iter().map(|ty| syn::parse_quote!(#ty: ::core::cmp::PartialEq));
    let where_clause = where_clause_with_bounds(where_clause, bounds);

    let eq_body = if field_idents.is_empty() {
        quote! { true }
    } else {
        quote! { true #(&& self.#field_idents == other.#field_idents)* }
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics ::core::cmp::PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                #eq_body
            }
        }
    }
}

fn eq_impl_tokens(
    name: &Ident,
    impl_generics: &impl quote::ToTokens,
    ty_generics: &impl quote::ToTokens,
    where_clause: Option<&syn::WhereClause>,
    fields: &FieldTokens,
) -> TokenStream {
    let FieldTokens { field_types, .. } = fields;
    let bounds = field_types.iter().map(|ty| syn::parse_quote!(#ty: ::core::cmp::Eq));
    let where_clause = where_clause_with_bounds(where_clause, bounds);

    quote! {
        #[automatically_derived]
        impl #impl_generics ::core::cmp::Eq for #name #ty_generics #where_clause {}
    }
}

fn hash_impl_tokens(
    name: &Ident,
    impl_generics: &impl quote::ToTokens,
    ty_generics: &impl quote::ToTokens,
    where_clause: Option<&syn::WhereClause>,
    fields: &FieldTokens,
) -> TokenStream {
    let FieldTokens { field_idents, field_types, .. } = fields;
    let bounds = field_types.iter().map(|ty| syn::parse_quote!(#ty: ::core::hash::Hash));
    let where_clause = where_clause_with_bounds(where_clause, bounds);

    quote! {
        #[automatically_derived]
        impl #impl_generics ::core::hash::Hash for #name #ty_generics #where_clause {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                #(self.#field_idents.hash(state);)*
            }
        }
    }
}

fn tagged_impl_tokens(
    name: &Ident,
    impl_generics: &impl quote::ToTokens,
    ty_generics: &impl quote::ToTokens,
    where_clause: Option<&syn::WhereClause>,
    tag_literal: &LitStr,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #impl_generics ::mhub_vault::Tagged for #name #ty_generics #where_clause {
            const TAG: &'static str = #tag_literal;
        }

        #[automatically_derived]
        impl #impl_generics ::mhub_vault::VaultSerde for #name #ty_generics #where_clause {}
    }
}

/// Expands the `#[vault_model]` macro.
pub fn expand_derive(args: TokenStream, input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut helper_generics = input.generics.clone();
    helper_generics.params.insert(0, parse_quote!('__mhub_vault_serde));
    let (helper_impl_generics, _helper_ty_generics, helper_where_clause) =
        helper_generics.split_for_impl();
    let helper_mod = format_ident!("__mhub_vault_derive_vault_{}", name);

    let tag_literal = match parse_tag_literal(args, &input) {
        Ok(lit) => lit,
        Err(err) => return err,
    };
    let derived_traits = derived_trait_names(&input);
    let fields = match named_fields(&input) {
        Ok(fields) => fields,
        Err(err) => return err,
    };
    let field_tokens = match build_field_tokens(fields, &helper_mod) {
        Ok(tokens) => tokens,
        Err(err) => return err,
    };

    let helper_mod_tokens = helper_mod_tokens(
        &helper_mod,
        &helper_impl_generics,
        helper_where_clause,
        &impl_generics,
        where_clause,
        &field_tokens,
    );
    let serde_tokens = serde_impl_tokens(
        name,
        &impl_generics,
        &ty_generics,
        where_clause,
        &helper_mod,
        &field_tokens,
    );
    let tagged_tokens =
        tagged_impl_tokens(name, &impl_generics, &ty_generics, where_clause, &tag_literal);
    let debug_tokens = if derived_traits.contains("Debug") {
        quote! {}
    } else {
        debug_impl_tokens(name, &impl_generics, &ty_generics, where_clause, &field_tokens)
    };
    let partial_eq_tokens = if derived_traits.contains("PartialEq") {
        quote! {}
    } else {
        partial_eq_impl_tokens(name, &impl_generics, &ty_generics, where_clause, &field_tokens)
    };
    let eq_tokens = if derived_traits.contains("Eq") {
        quote! {}
    } else {
        eq_impl_tokens(name, &impl_generics, &ty_generics, where_clause, &field_tokens)
    };
    let hash_tokens = if derived_traits.contains("Hash") {
        quote! {}
    } else {
        hash_impl_tokens(name, &impl_generics, &ty_generics, where_clause, &field_tokens)
    };

    quote! {
        #input
        #helper_mod_tokens
        #serde_tokens
        #tagged_tokens
        #debug_tokens
        #partial_eq_tokens
        #eq_tokens
        #hash_tokens
    }
}
