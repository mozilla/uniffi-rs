/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::convert::{Into, TryFrom};
use syn::spanned::Spanned;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDefinition {}

impl MethodDefinition {}

impl TryFrom<&syn::ItemImpl> for MethodDefinition {
    type Error = syn::Error;
    fn try_from(_item: &syn::ItemImpl) -> syn::Result<Self> {
        Ok(MethodDefinition {})
    }
}

impl Into<proc_macro2::TokenStream> for &MethodDefinition {
    fn into(self) -> proc_macro2::TokenStream {
        quote! {
            // Not implemented yet...
        }
    }
}

impl quote::ToTokens for MethodDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tt: proc_macro2::TokenStream = self.into();
        tt.to_tokens(tokens);
    }
}

// We expect to parse multiple method definitions from a single `ItemImpl` block.

pub struct MethodDefinitions {
    methods: Vec<MethodDefinition>,
}

impl TryFrom<&syn::ItemImpl> for MethodDefinitions {
    type Error = syn::Error;
    fn try_from(_item: &syn::ItemImpl) -> syn::Result<Self> {
        Ok(MethodDefinitions { methods: vec![] })
    }
}

impl Into<proc_macro2::TokenStream> for &MethodDefinitions {
    fn into(self) -> proc_macro2::TokenStream {
        quote! {
            // Not implemented yet...
        }
    }
}

impl quote::ToTokens for MethodDefinitions {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tt: proc_macro2::TokenStream = self.into();
        tt.to_tokens(tokens);
    }
}
