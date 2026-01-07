use fxhash::FxHashSet;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{Attribute, ItemFn, ItemStruct, Lit, LitStr, Meta};

/// Expands the `#[api_model]` attribute macro.
///
/// Automatically adds common derives (`Serialize`, `Deserialize`, `ToSchema`) and
/// configures Serde for camelCase and strict field checking.
pub fn expand_api_model(args: TokenStream, input: ItemStruct) -> TokenStream {
    let ApiModelArgs { rename_all, deny_unknown_fields } = match parse_api_model_args(args) {
        Ok(args) => args,
        Err(err) => return err,
    };
    let derives = derived_trait_names(&input.attrs);
    let serde_meta = match serde_meta_info(&input.attrs) {
        Ok(info) => info,
        Err(err) => return err,
    };

    let derive_attr = derive_attr(&derives);
    let to_schema_attr = to_schema_attr(&derives);

    let rename_attr = match rename_attr(rename_all, &serde_meta) {
        Ok(attr) => attr,
        Err(err) => return err,
    };
    let deny_attr = match deny_unknown_attr(deny_unknown_fields, &serde_meta, &input) {
        Ok(attr) => attr,
        Err(err) => return err,
    };

    quote! {
        #derive_attr
        #to_schema_attr
        #rename_attr
        #deny_attr
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
        #[cfg_attr(feature = "server", ::utoipa::path(#args))]
        #vis #sig {
            #body
        }
    }
}

struct ApiModelArgs {
    rename_all: Option<LitStr>,
    deny_unknown_fields: Option<bool>,
}

fn parse_api_model_args(args: TokenStream) -> Result<ApiModelArgs, TokenStream> {
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let metas = parser.parse2(args).map_err(|err| err.to_compile_error())?;

    let mut rename_all = None;
    let mut deny_unknown_fields = None;

    for meta in metas {
        let name_value = expect_name_value(meta)?;
        if name_value.path.is_ident("rename_all") {
            let value = parse_string_literal(&name_value, "rename_all")?;
            rename_all = Some(set_once(rename_all, &name_value, value)?);
            continue;
        }
        if name_value.path.is_ident("deny_unknown_fields") {
            let value = parse_bool_literal(&name_value, "deny_unknown_fields")?;
            deny_unknown_fields = Some(set_once(deny_unknown_fields, &name_value, value)?);
            continue;
        }
        return Err(syn::Error::new_spanned(
            name_value.path,
            "Unsupported argument; expected rename_all or deny_unknown_fields",
        )
        .to_compile_error());
    }

    Ok(ApiModelArgs { rename_all, deny_unknown_fields })
}

fn expect_name_value(meta: Meta) -> Result<syn::MetaNameValue, TokenStream> {
    match meta {
        Meta::NameValue(name_value) => Ok(name_value),
        other => Err(syn::Error::new_spanned(
            other,
            "Expected name-value arguments like `rename_all = \"...\"`",
        )
        .to_compile_error()),
    }
}

fn parse_bool_literal(name_value: &syn::MetaNameValue, label: &str) -> Result<bool, TokenStream> {
    match &name_value.value {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Bool(lit) => Ok(lit.value),
            _ => Err(syn::Error::new_spanned(
                &name_value.value,
                format!("{label} must be a boolean literal"),
            )
            .to_compile_error()),
        },
        _ => Err(syn::Error::new_spanned(
            &name_value.value,
            format!("{label} must be a boolean literal"),
        )
        .to_compile_error()),
    }
}

fn parse_string_literal(
    name_value: &syn::MetaNameValue,
    label: &str,
) -> Result<LitStr, TokenStream> {
    match &name_value.value {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(lit) => Ok(lit.clone()),
            _ => Err(syn::Error::new_spanned(
                &name_value.value,
                format!("{label} must be a string literal"),
            )
            .to_compile_error()),
        },
        _ => Err(syn::Error::new_spanned(
            &name_value.value,
            format!("{label} must be a string literal"),
        )
        .to_compile_error()),
    }
}

fn set_once<T>(current: Option<T>, token: &syn::MetaNameValue, value: T) -> Result<T, TokenStream> {
    if current.is_some() {
        return Err(syn::Error::new_spanned(token, "Duplicate argument").to_compile_error());
    }
    Ok(value)
}

struct SerdeMetaInfo {
    rename_all: Option<LitStr>,
    deny_unknown_fields: bool,
}

fn derive_attr(derives: &FxHashSet<String>) -> TokenStream {
    let mut tokens = Vec::new();
    if !derives.contains("Debug") {
        tokens.push(quote! { Debug });
    }
    if !derives.contains("Serialize") {
        tokens.push(quote! { ::serde::Serialize });
    }
    if !derives.contains("Deserialize") {
        tokens.push(quote! { ::serde::Deserialize });
    }

    if tokens.is_empty() {
        quote! {}
    } else {
        quote! { #[derive(#(#tokens),*)] }
    }
}

fn to_schema_attr(derives: &FxHashSet<String>) -> TokenStream {
    if derives.contains("ToSchema") {
        quote! {}
    } else {
        quote! { #[cfg_attr(feature = "server", derive(::utoipa::ToSchema))] }
    }
}

fn rename_attr(
    rename_all: Option<LitStr>,
    serde_meta: &SerdeMetaInfo,
) -> Result<TokenStream, TokenStream> {
    let rename_all_value =
        rename_all.unwrap_or_else(|| LitStr::new("camelCase", proc_macro2::Span::call_site()));

    match &serde_meta.rename_all {
        Some(existing) if existing.value() != rename_all_value.value() => Err(
            syn::Error::new_spanned(
                existing,
                "Conflicting serde rename_all; remove it or set api_model(rename_all = \"...\") to match",
            )
            .to_compile_error(),
        ),
        Some(_) => Ok(quote! {}),
        None => Ok(quote! { #[serde(rename_all = #rename_all_value)] }),
    }
}

fn deny_unknown_attr(
    deny_unknown_fields: Option<bool>,
    serde_meta: &SerdeMetaInfo,
    input: &ItemStruct,
) -> Result<TokenStream, TokenStream> {
    let deny_unknown = deny_unknown_fields.unwrap_or(true);
    if serde_meta.deny_unknown_fields {
        if !deny_unknown {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "deny_unknown_fields is already set via serde; remove it before disabling",
            )
            .to_compile_error());
        }
        return Ok(quote! {});
    }

    if deny_unknown { Ok(quote! { #[serde(deny_unknown_fields)] }) } else { Ok(quote! {}) }
}

fn serde_meta_info(attrs: &[Attribute]) -> Result<SerdeMetaInfo, TokenStream> {
    let mut rename_all = None;
    let mut deny_unknown_fields = false;

    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let res = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                rename_all = Some(lit);
                return Ok(());
            }
            if meta.path.is_ident("deny_unknown_fields") {
                deny_unknown_fields = true;
                return Ok(());
            }
            Ok(())
        });

        if let Err(err) = res {
            return Err(err.to_compile_error());
        }
    }

    Ok(SerdeMetaInfo { rename_all, deny_unknown_fields })
}

fn derived_trait_names(attrs: &[Attribute]) -> FxHashSet<String> {
    let mut traits = FxHashSet::default();

    for attr in attrs {
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
