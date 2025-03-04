/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod checksum;
mod from_uniffi_meta;
mod ir;

/// Custom derive for uniffi_meta::Checksum
#[proc_macro_derive(Checksum, attributes(checksum_ignore))]
pub fn checksum_derive(input: TokenStream) -> TokenStream {
    checksum::expand_derive(parse_macro_input!(input)).into()
}

/// Derive From<uniffi_meta::MetadataType>
#[proc_macro_derive(FromUniffiMeta, attributes(from))]
pub fn from_uniffi_meta_derive(input: TokenStream) -> TokenStream {
    from_uniffi_meta::expand_derive(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn ir(input: TokenStream) -> TokenStream {
    ir::expand_ir(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn ir_pass(input: TokenStream) -> TokenStream {
    ir::expand_ir_pass(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn define_ir_pass(input: TokenStream) -> TokenStream {
    ir::expand_define_ir_pass(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Node, attributes(from_uniffi_meta))]
pub fn node(input: TokenStream) -> TokenStream {
    ir::expand_node(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn construct_node(input: TokenStream) -> TokenStream {
    ir::expand_construct_node(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
