use super::parsing::ErrorVariantInfo;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

pub(super) fn generate_implementations(
    enum_name: &Ident,
    infos: &[ErrorVariantInfo],
) -> TokenStream {
    let data_struct_def = generate_data_struct(enum_name);
    let error_metadata_impl = generate_error_metadata(enum_name, infos);
    let display_impl = generate_display(enum_name);
    let std_error_impl = generate_std_error(enum_name, infos);
    let constructors = generate_constructors(enum_name, infos);
    let mutators = generate_mutators(enum_name, infos);
    let trait_def_and_impl = generate_trait_ext(enum_name);
    let source_impls = generate_source_impls(enum_name, infos);
    let from_impls = generate_from_impls(enum_name, infos);
    let helpers = generate_helpers();

    quote! {
        #data_struct_def
        #constructors
        #mutators
        #std_error_impl
        #error_metadata_impl
        #display_impl
        #trait_def_and_impl
        #source_impls
        #from_impls
        #helpers
    }
}

fn generate_data_struct(enum_name: &Ident) -> TokenStream {
    let data_name = format_ident!("{}Data", enum_name);
    quote! {
        #[doc(hidden)]
        #[derive(Debug)]
        pub struct #data_name {
            pub message: ::std::borrow::Cow<'static, str>,
            pub code: ErrorCode,
            pub kind: ::std::borrow::Cow<'static, str>,
            pub context: Option<::std::borrow::Cow<'static, str>>,
            pub help: Option<::std::borrow::Cow<'static, str>>,
            pub backtrace: Option<::std::backtrace::Backtrace>,
        }
    }
}

fn generate_constructors(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let data_name = format_ident!("{enum_name}Data");

    let methods = infos.iter().filter(|i| i.source.is_none()).map(|info| {
        let variant_ident = &info.ident;
        let fn_name = format_ident!("{}", variant_ident.to_string().to_snake_case());
        let msg = &info.message;
        let code = &info.code;
        let kind = &info.kind;
        let cfg = &info.cfg_attrs;
        let help = info
            .help
            .as_ref()
            .map_or_else(|| quote! { None }, |h| quote! { Some(::std::borrow::Cow::Borrowed(#h)) });

        quote! {
            #(#cfg)*
            pub fn #fn_name() -> Self {
                Self::#variant_ident {
                    data: Box::new(#data_name {
                        message: #msg.into(),
                        code: #code.into(),
                        kind: #kind.into(),
                        context: None,
                        help: #help,
                        backtrace: backtrace(),
                    })
                }
            }
        }
    });

    quote! {
        impl #enum_name {
            #(#methods)*
        }
    }
}

fn generate_mutators(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let message_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.message = message.into(), }
    });
    let code_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.code = code.into(), }
    });
    let kind_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.kind = kind.into(), }
    });
    let context_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.context = Some(context.into()), }
    });
    let help_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.help = Some(help.into()), }
    });

    quote! {
        impl #enum_name {
            pub fn with_message(mut self, message: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                match &mut self { #(#message_arms)* }
                self
            }
            pub fn with_code(mut self, code: impl Into<ErrorCode>) -> Self {
                match &mut self { #(#code_arms)* }
                self
            }
            pub fn with_kind(mut self, kind: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                match &mut self { #(#kind_arms)* }
                self
            }
            pub fn with_context(mut self, context: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                match &mut self { #(#context_arms)* }
                self
            }
            pub fn with_context_fn<F>(mut self, f: F) -> Self where F: FnOnce() -> String {
                self.with_context(f())
            }
            pub fn with_help(mut self, help: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                match &mut self { #(#help_arms)* }
                self
            }
            pub fn with_help_fn<F>(mut self, f: F) -> Self where F: FnOnce() -> String {
                self.with_help(f())
            }
        }
    }
}

fn generate_std_error(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let match_arms = infos.iter().map(|info| {
        let variant = &info.ident;
        let cfg = &info.cfg_attrs;
        if info.source.is_some() {
            quote! { #(#cfg)* #enum_name::#variant { source, .. } => Some(source), }
        } else {
            quote! { #(#cfg)* #enum_name::#variant { .. } => None, }
        }
    });

    quote! {
        impl ::std::error::Error for #enum_name {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

fn generate_error_metadata(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let message_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.message.clone(), }
    });
    let code_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.code, }
    });
    let kind_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.kind.clone(), }
    });
    let context_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.context.as_ref(), }
    });
    let help_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.help.as_ref(), }
    });
    let backtrace_arms = infos.iter().map(|i| {
        let v = &i.ident;
        let cfg = &i.cfg_attrs;
        quote! { #(#cfg)* Self::#v { data, .. } => data.backtrace.as_ref(), }
    });

    quote! {
        impl ErrorMetadata for #enum_name {
            fn message(&self) -> ::std::borrow::Cow<'static, str> {
                match self { #(#message_arms)* }
            }
            fn code(&self) -> ErrorCode {
                match self { #(#code_arms)* }
            }
            fn kind(&self) -> ::std::borrow::Cow<'static, str> {
                match self { #(#kind_arms)* }
            }
            fn context(&self) -> Option<&::std::borrow::Cow<'static, str>> {
                match self { #(#context_arms)* }
            }
            fn help(&self) -> Option<&::std::borrow::Cow<'static, str>> {
                match self { #(#help_arms)* }
            }
            fn backtrace(&self) -> Option<&::std::backtrace::Backtrace> {
                match self { #(#backtrace_arms)* }
            }
            fn target(&self) -> &'static str {
                env!("CARGO_PKG_NAME")
            }
        }
    }
}

fn generate_display(enum_name: &Ident) -> TokenStream {
    quote! {
        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let message = self.message();
                let context = self.context().as_ref().map(|c| format!(" ({c})")).unwrap_or_default();

                write!(f, "{message}{context}")
            }
        }
    }
}

fn generate_trait_ext(enum_name: &Ident) -> TokenStream {
    let trait_name = format_ident!("{}Ext", enum_name);
    quote! {
        pub trait #trait_name<T> {
            fn with_message(self, message: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name>;
            fn with_code(self, code: impl Into<ErrorCode>) -> ::std::result::Result<T, #enum_name>;
            fn with_kind(self, kind: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name>;
            fn with_context(self, context: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name>;
            fn with_context_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String;
            fn with_help(self, help: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name>;
            fn with_help_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String;
        }
    }
}

fn generate_from_impls(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let data_name = format_ident!("{enum_name}Data");

    let impls = infos.iter().filter(|i| i.source.is_some()).map(|info| {
        let variant_ident = &info.ident;
        let source_type = info.source.as_ref().unwrap();
        let msg = &info.message;
        let code = &info.code;
        let kind = &info.kind;
        let cfg = &info.cfg_attrs;

        let help = info.help.as_ref().map_or_else(
            || quote! { None },
            |h| quote! { Some(::std::borrow::Cow::Borrowed(#h)) }
        );

        quote! {
            #(#cfg)*
            impl ::std::convert::From<#source_type> for #enum_name {
                fn from(source: #source_type) -> Self {
                    let backtrace = if ::std::env::var("RUST_BACKTRACE").map_or(false, |v| v != "0") {
                        Some(::std::backtrace::Backtrace::capture())
                    } else {
                        None
                    };

                    Self::#variant_ident {
                        source: Box::new(source),
                        data: Box::new(#data_name {
                            message: #msg.into(),
                            code: #code.into(),
                            kind: #kind.into(),
                            context: None,
                            help: #help,
                            backtrace,
                        })
                    }
                }
            }
        }
    });

    quote! { #(#impls)* }
}

#[allow(clippy::too_many_lines)]
fn generate_source_impls(enum_name: &Ident, infos: &[ErrorVariantInfo]) -> TokenStream {
    let mut stream = TokenStream::new();
    let trait_name = format_ident!("{enum_name}Ext");

    let main_ext_impl = quote! {
        impl<T> #trait_name<T> for ::std::result::Result<T, #enum_name> {
            fn with_message(self, message: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                self.map_err(|e| e.with_message(message))
            }
            fn with_code(self, code: impl Into<ErrorCode>) -> Self {
                self.map_err(|e| e.with_code(code))
            }
            fn with_kind(self, kind: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                self.map_err(|e| e.with_kind(kind))
            }
            fn with_context(self, context: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                self.map_err(|e| e.with_context(context))
            }
            fn with_context_fn<F>(self, f: F) -> Self where F: FnOnce() -> String {
                self.map_err(|e| e.with_context_fn(f))
            }
            fn with_help(self, help: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                self.map_err(|e| e.with_help(help))
            }
            fn with_help_fn<F>(self, f: F) -> Self where F: FnOnce() -> String {
                self.map_err(|e| e.with_help_fn(f))
            }
        }
    };
    stream.extend(main_ext_impl);

    for info in infos.iter().filter(|i| i.source.is_some()) {
        let source_type = info.source.as_ref().unwrap();
        let cfg = &info.cfg_attrs;

        let foreign_ext_impl = quote! {
            #(#cfg)*
            impl<T> #trait_name<T> for ::std::result::Result<T, #source_type> {
                fn with_message(self, message: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name> {
                    self.map_err(#enum_name::from).map_err(|e| e.with_message(message))
                }
                fn with_code(self, code: impl Into<ErrorCode>) -> ::std::result::Result<T, #enum_name> {
                    self.map_err(#enum_name::from).map_err(|e| e.with_code(code))
                }
                fn with_kind(self, kind: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name> {
                    self.map_err(#enum_name::from).map_err(|e| e.with_kind(kind))
                }
                fn with_context(self, context: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name> {
                    self.map_err(#enum_name::from).map_err(|e| e.with_context(context))
                }
                fn with_context_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String {
                    self.map_err(#enum_name::from).map_err(|e| e.with_context_fn(f))
                }
                fn with_help(self, help: impl Into<::std::borrow::Cow<'static, str>>) -> ::std::result::Result<T, #enum_name> {
                    self.map_err(#enum_name::from).map_err(|e| e.with_help(help))
                }
                fn with_help_fn<F>(self, f: F) -> ::std::result::Result<T, #enum_name> where F: FnOnce() -> String {
                    self.map_err(#enum_name::from).map_err(|e| e.with_help_fn(f))
                }
            }
        };
        stream.extend(foreign_ext_impl);
    }

    stream
}

#[inline]
fn generate_helpers() -> TokenStream {
    quote! {
        #[inline]
        fn backtrace() -> Option<::std::backtrace::Backtrace> {
            if ::std::env::var("RUST_BACKTRACE").map_or(false, |v| v != "0") {
                Some(::std::backtrace::Backtrace::capture())
            } else {
                None
            }
        }
    }
}
