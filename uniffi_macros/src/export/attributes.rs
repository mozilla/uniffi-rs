use crate::util::{either_attribute_arg, parse_comma_separated, UniffiAttributeArgs};

use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    LitStr, Token,
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
    Tokio(Span),
}

impl Parse for AsyncRuntime {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;
        match lit.value().as_str() {
            "tokio" => Ok(Self::Tokio(lit.span())),
            _ => Err(syn::Error::new_spanned(
                lit,
                "unknown async runtime, currently only `tokio` is supported",
            )),
        }
    }
}

impl Spanned for AsyncRuntime {
    fn span(&self) -> Span {
        match self {
            AsyncRuntime::Tokio(span) => *span,
        }
    }
}
