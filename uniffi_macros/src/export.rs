/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    LitStr, Token,
};

pub(crate) mod metadata;
mod scaffolding;

pub use self::metadata::gen_metadata;
use self::scaffolding::{gen_fn_scaffolding, gen_method_scaffolding};
use crate::util::{either_attribute_arg, parse_comma_separated, UniffiAttribute};

// TODO(jplatte): Ensure no generics, …
// TODO(jplatte): Aggregate errors instead of short-circuiting, wherever possible

pub enum ExportItem {
    Function {
        sig: Signature,
    },
    Impl {
        self_ident: Ident,
        methods: Vec<syn::Result<Method>>,
    },
}

pub struct Method {
    sig: Signature,
}

pub struct Signature {
    ident: Ident,
    is_async: bool,
    inputs: Vec<syn::FnArg>,
    output: TokenStream,
}

impl Signature {
    fn new(item: syn::Signature) -> syn::Result<Self> {
        let output = match item.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };

        Ok(Self {
            ident: item.ident,
            is_async: item.asyncness.is_some(),
            inputs: item.inputs.into_iter().collect(),
            output,
        })
    }
}

pub fn expand_export(
    metadata: ExportItem,
    arguments: ExportAttributeArguments,
    mod_path: &str,
) -> syn::Result<TokenStream> {
    match metadata {
        ExportItem::Function { sig } => gen_fn_scaffolding(&sig, mod_path, &arguments),
        ExportItem::Impl {
            methods,
            self_ident,
        } => {
            let mut method_tokens = vec![];
            for method in methods {
                let sig = method?.sig;
                method_tokens.push(gen_method_scaffolding(
                    &sig,
                    mod_path,
                    &self_ident,
                    &arguments,
                )?)
            }
            Ok(quote_spanned! { self_ident.span() => #(#method_tokens)* })
        }
    }
}

mod kw {
    syn::custom_keyword!(async_runtime);
}

#[derive(Default)]
pub struct ExportAttributeArguments {
    async_runtime: Option<AsyncRuntime>,
}

impl Parse for ExportAttributeArguments {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}

impl UniffiAttribute for ExportAttributeArguments {
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

enum AsyncRuntime {
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
