/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::convert::{Into, TryFrom};
use std::matches;
use syn::spanned::Spanned;

use crate::syn_err;

// Represents a simple C-style enum.
// In the FFI these are turned into a plain u32.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDefinition {
    name: String,
    values: Vec<String>,
}

impl EnumDefinition {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn values(&self) -> Vec<&str> {
        self.values.iter().map(|v| v.as_str()).collect()
    }
}

impl TryFrom<&syn::ItemEnum> for EnumDefinition {
    type Error = syn::Error;
    fn try_from(item: &syn::ItemEnum) -> syn::Result<Self> {
        if !matches!(item.vis, syn::Visibility::Public(_)) {
            return syn_err!(item, "Exported enums must be public");
        }
        if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
            return syn_err!(item.generics, "Exported enums cannot have generics");
        }
        // XXX TODO: check valid names, reserved words etc.
        Ok(EnumDefinition {
            name: item.ident.to_string(),
            values: item
                .variants
                .iter()
                .map(|v| {
                    if !matches!(v.fields, syn::Fields::Unit) {
                        return syn_err!(v, "Exported enum variants cannot have fields");
                    }
                    if v.discriminant.is_some() {
                        return syn_err!(
                            v,
                            "Exported enum fields cannot have explicit discriminant"
                        );
                    }
                    Ok(v.ident.to_string())
                })
                .collect::<syn::Result<Vec<_>>>()?,
        })
    }
}

impl Into<proc_macro2::TokenStream> for &EnumDefinition {
    fn into(self) -> proc_macro2::TokenStream {
        let name = format_ident!("{}", self.name);
        let name_str = self.name.as_str();
        let variants = self
            .values
            .iter()
            .map(|v| format_ident!("{}", v))
            .collect::<Vec<_>>();
        let indices = (1..self.values.len() + 1)
            .map(|i| syn::Index::from(i))
            .collect::<Vec<_>>();
        let serialized_defn = super::ExportDefinition::Enum(self.clone());
        quote! {
            unsafe impl uniffi::support::ViaFfi for #name {
                type Value = u32;
                const NAME: &'static str = #name_str;
                fn into_ffi_value(self) -> Self::Value {
                    match self {
                        #(#name::#variants => #indices),*
                    }
                }
                fn try_from_ffi_value(v: Self::Value) -> anyhow::Result<Self> {
                    Ok(match v {
                        #(#indices => #name::#variants),*,
                        _ => anyhow::bail!("Invalid {} enum value: {}", Self::NAME, v),
                    })
                }
            }
            #serialized_defn
        }
    }
}

impl quote::ToTokens for EnumDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tt: proc_macro2::TokenStream = self.into();
        tt.to_tokens(tokens);
    }
}
