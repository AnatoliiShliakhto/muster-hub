use super::parsing::ErrorVariantInfo;
use syn::{Fields, FieldsNamed, ItemEnum, parse_quote};

pub(super) fn transform_enum_variants(item: &mut ItemEnum, infos: &[ErrorVariantInfo]) {
    let data_struct_name = quote::format_ident!("{}Data", item.ident);

    for (variant, info) in item.variants.iter_mut().zip(infos.iter()) {
        let source_field = info
            .source
            .as_ref()
            .map_or_else(|| quote::quote! {}, |src_type| quote::quote! { source: Box<#src_type>, });

        let fields: FieldsNamed = parse_quote! {
            {
                #source_field
                data: Box<#data_struct_name>,
            }
        };

        variant.fields = Fields::Named(fields);
    }
}
