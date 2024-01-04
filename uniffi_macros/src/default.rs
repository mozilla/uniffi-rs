/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::util::kw;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Lit,
};

/// Default value
#[derive(Clone)]
pub enum DefaultValue {
    Literal(Lit),
    Null(kw::None),
}

impl ToTokens for DefaultValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DefaultValue::Literal(lit) => lit.to_tokens(tokens),
            DefaultValue::Null(kw) => kw.to_tokens(tokens),
        }
    }
}

impl Parse for DefaultValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::None) {
            let none_kw: kw::None = input.parse()?;
            Ok(Self::Null(none_kw))
        } else {
            Ok(Self::Literal(input.parse()?))
        }
    }
}

impl DefaultValue {
    fn metadata_calls(&self) -> syn::Result<TokenStream> {
        match self {
            DefaultValue::Literal(Lit::Int(i)) if !i.suffix().is_empty() => Err(
                syn::Error::new_spanned(i, "integer literals with suffix not supported here"),
            ),
            DefaultValue::Literal(Lit::Float(f)) if !f.suffix().is_empty() => Err(
                syn::Error::new_spanned(f, "float literals with suffix not supported here"),
            ),

            DefaultValue::Literal(Lit::Str(s)) => Ok(quote! {
                .concat_value(::uniffi::metadata::codes::LIT_STR)
                .concat_str(#s)
            }),
            DefaultValue::Literal(Lit::Int(i)) => {
                let digits = i.base10_digits();
                Ok(quote! {
                    .concat_value(::uniffi::metadata::codes::LIT_INT)
                    .concat_str(#digits)
                })
            }
            DefaultValue::Literal(Lit::Float(f)) => {
                let digits = f.base10_digits();
                Ok(quote! {
                    .concat_value(::uniffi::metadata::codes::LIT_FLOAT)
                    .concat_str(#digits)
                })
            }
            DefaultValue::Literal(Lit::Bool(b)) => Ok(quote! {
                .concat_value(::uniffi::metadata::codes::LIT_BOOL)
                .concat_bool(#b)
            }),

            DefaultValue::Literal(_) => Err(syn::Error::new_spanned(
                self,
                "this type of literal is not currently supported as a default",
            )),

            DefaultValue::Null(_) => Ok(quote! {
                .concat_value(::uniffi::metadata::codes::LIT_NULL)
            }),
        }
    }
}

pub fn default_value_metadata_calls(default: &Option<DefaultValue>) -> syn::Result<TokenStream> {
    Ok(match default {
        Some(default) => {
            let metadata_calls = default.metadata_calls()?;
            quote! {
                .concat_bool(true)
                #metadata_calls
            }
        }
        None => quote! { .concat_bool(false) },
    })
}
