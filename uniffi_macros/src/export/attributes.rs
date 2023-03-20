use crate::util::{either_attribute_arg, parse_comma_separated, UniffiAttributeArgs};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, LitStr, Meta, PathArguments, PathSegment, Token,
};

pub(crate) mod kw {
    syn::custom_keyword!(async_runtime);
}

#[derive(Default)]
pub struct ExportAttributeArguments {
    pub(crate) async_runtime: Option<AsyncRuntime>,
}

impl Parse for ExportAttributeArguments {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}

impl UniffiAttributeArgs for ExportAttributeArguments {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::async_runtime = input.parse()?;
        let _: Token![=] = input.parse()?;
        let async_runtime = input.parse()?;
        Ok(Self {
            async_runtime: Some(async_runtime),
        })
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            async_runtime: either_attribute_arg(self.async_runtime, other.async_runtime)?,
        })
    }
}

pub(crate) enum AsyncRuntime {
    Tokio(LitStr),
}

impl Parse for AsyncRuntime {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;
        match lit.value().as_str() {
            "tokio" => Ok(Self::Tokio(lit)),
            _ => Err(syn::Error::new_spanned(
                lit,
                "unknown async runtime, currently only `tokio` is supported",
            )),
        }
    }
}

impl ToTokens for AsyncRuntime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            AsyncRuntime::Tokio(lit) => lit.to_tokens(tokens),
        }
    }
}

#[derive(Default)]
pub(super) struct ExportedImplFnAttributes {
    pub constructor: bool,
}

impl ExportedImplFnAttributes {
    pub fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = Self::default();
        for attr in attrs {
            let segs = &attr.path().segments;

            let fst = segs
                .first()
                .expect("attributes have at least one path segment");
            if fst.ident != "uniffi" {
                continue;
            }
            ensure_no_path_args(fst)?;

            if let Meta::List(_) | Meta::NameValue(_) = &attr.meta {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "attribute arguments are not currently recognized in this position",
                ));
            }

            if segs.len() != 2 {
                return Err(syn::Error::new_spanned(
                    segs,
                    "unsupported uniffi attribute",
                ));
            }
            let snd = &segs[1];
            ensure_no_path_args(snd)?;

            match snd.ident.to_string().as_str() {
                "constructor" => {
                    if this.constructor {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "duplicate constructor attribute",
                        ));
                    }
                    this.constructor = true;
                }
                _ => return Err(syn::Error::new_spanned(snd, "unknown uniffi attribute")),
            }
        }

        Ok(this)
    }
}

fn ensure_no_path_args(seg: &PathSegment) -> syn::Result<()> {
    if matches!(seg.arguments, PathArguments::None) {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(&seg.arguments, "unexpected syntax"))
    }
}
