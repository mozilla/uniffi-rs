/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod checksum;
mod map_node;
mod node;

/// Custom derive for uniffi_meta::Checksum
#[proc_macro_derive(Checksum, attributes(checksum_ignore))]
pub fn checksum_derive(input: TokenStream) -> TokenStream {
    checksum::expand_derive(parse_macro_input!(input)).into()
}

#[proc_macro_derive(Node)]
pub fn node(input: TokenStream) -> TokenStream {
    node::expand_derive(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(MapNode, attributes(map_node))]
pub fn map_node(input: TokenStream) -> TokenStream {
    map_node::expand_derive(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn use_prev_node(input: TokenStream) -> TokenStream {
    map_node::expand_use_prev_node(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
