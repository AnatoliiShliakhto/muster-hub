use heck::{ToShoutySnakeCase, ToSnakeCase};
use syn::{Attribute, ItemEnum, LitStr, Result, Type, parse_quote};

pub(super) struct ErrorVariantInfo {
    pub ident: syn::Ident,
    pub source: Option<Type>,
    pub message: String,
    pub code: syn::Expr,
    pub kind: String,
    pub help: Option<String>,
    pub cfg_attrs: Vec<Attribute>,
}

pub(super) fn parse_and_clean_variants(item: &mut ItemEnum) -> Result<Vec<ErrorVariantInfo>> {
    let mut infos = Vec::new();

    for variant in &mut item.variants {
        let cfg_attrs: Vec<_> =
            variant.attrs.iter().filter(|a| a.path().is_ident("cfg")).cloned().collect();

        let mut info = ErrorVariantInfo {
            ident: variant.ident.clone(),
            source: None,
            message: variant.ident.to_string().to_snake_case().replace('_', " "),
            code: parse_quote!(500),
            kind: variant.ident.to_string().to_shouty_snake_case(),
            help: None,
            cfg_attrs,
        };

        let mut attr_to_remove = None;

        for (i, attr) in variant.attrs.iter().enumerate() {
            if attr.path().is_ident("error") {
                parse_error_attribute(attr, &mut info)?;
                attr_to_remove = Some(i);
                break;
            }
        }

        if let Some(i) = attr_to_remove {
            variant.attrs.remove(i);
        }

        infos.push(info);
    }

    Ok(infos)
}

fn parse_error_attribute(attr: &Attribute, info: &mut ErrorVariantInfo) -> Result<()> {
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("code") {
            info.code = meta.value()?.parse::<syn::Expr>()?;
            Ok(())
        } else if meta.path.is_ident("message") || meta.path.is_ident("msg") {
            let content: LitStr = meta.value()?.parse()?;
            info.message = content.value();
            Ok(())
        } else if meta.path.is_ident("kind")
            || meta.path.is_ident("slug")
            || meta.path.is_ident("type")
        {
            let content: LitStr = meta.value()?.parse()?;
            info.kind = content.value().to_shouty_snake_case();
            Ok(())
        } else if meta.path.is_ident("source") || meta.path.is_ident("src") {
            let ty: Type = meta.value()?.parse()?;
            info.source = Some(ty);
            Ok(())
        } else if meta.path.is_ident("help")
            || meta.path.is_ident("hlp")
            || meta.path.is_ident("info")
        {
            let help: LitStr = meta.value()?.parse()?;
            info.help = Some(help.value());
            Ok(())
        } else {
            Err(meta.error("unsupported property in #[error]"))
        }
    })
}
