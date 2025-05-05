/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod as_ref;
mod checksum;
mod ir;

/// Custom derive for uniffi_meta::Checksum
#[proc_macro_derive(Checksum, attributes(checksum_ignore))]
pub fn checksum_derive(input: TokenStream) -> TokenStream {
    checksum::expand_derive(parse_macro_input!(input)).into()
}

#[proc_macro_derive(Node, attributes(node))]
pub fn node(input: TokenStream) -> TokenStream {
    ir::expand_node(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(AsRef, attributes(as_ref))]
pub fn expand_as_ref(input: TokenStream) -> TokenStream {
    as_ref::expand_as_ref(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
