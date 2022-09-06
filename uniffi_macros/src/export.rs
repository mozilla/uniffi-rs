/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use uniffi_meta::{checksum, FnMetadata, Metadata, Type};

mod metadata;
mod scaffolding;

use self::{metadata::gen_metadata, scaffolding::gen_fn_scaffolding};

// TODO(jplatte): Ensure no generics, no async, …
// TODO(jplatte): Aggregate errors instead of short-circuiting, whereever possible

enum ExportItem {
    Function {
        item: syn::ItemFn,
        metadata: FnMetadata,
    },
}

pub fn expand_export(item: syn::Item, mod_path: &[String]) -> syn::Result<TokenStream> {
    match gen_metadata(item, mod_path)? {
        ExportItem::Function { item, metadata } => {
            let checksum = checksum(&metadata);
            let scaffolding = gen_fn_scaffolding(&item, mod_path, checksum)?;
            let type_assertions = fn_type_assertions(&item);
            let meta_static_var = create_metadata_static_var(&item.sig.ident, metadata.into());

            Ok(quote! {
                #scaffolding
                #type_assertions
                #meta_static_var
            })
        }
    }
}

fn fn_type_assertions(item: &syn::ItemFn) -> TokenStream {
    // Convert uniffi_meta::Type back to a Rust type
    fn convert_type_back(ty: &Type) -> TokenStream {
        match &ty {
            Type::U8 => quote! { ::std::primitive::u8 },
            Type::U16 => quote! { ::std::primitive::u16 },
            Type::U32 => quote! { ::std::primitive::u32 },
            Type::U64 => quote! { ::std::primitive::u64 },
            Type::I8 => quote! { ::std::primitive::i8 },
            Type::I16 => quote! { ::std::primitive::i16 },
            Type::I32 => quote! { ::std::primitive::i32 },
            Type::I64 => quote! { ::std::primitive::i64 },
            Type::F32 => quote! { ::std::primitive::f32 },
            Type::F64 => quote! { ::std::primitive::f64 },
            Type::Bool => quote! { ::std::primitive::bool },
            Type::String => quote! { ::std::string::String },
            Type::Option { inner_type } => {
                let inner = convert_type_back(inner_type);
                quote! { ::std::option::Option<#inner> }
            }
            Type::Vec { inner_type } => {
                let inner = convert_type_back(inner_type);
                quote! { ::std::vec::Vec<#inner> }
            }
            Type::HashMap {
                key_type,
                value_type,
            } => {
                let key = convert_type_back(key_type);
                let value = convert_type_back(value_type);
                quote! { ::std::collections::HashMap<#key, #value> }
            }
            Type::ArcObject { object_name } => {
                let object_ident = format_ident!("{object_name}");
                quote! { ::std::sync::Arc<crate::uniffi_types::#object_ident> }
            }
        }
    }

    let input_types = item.sig.inputs.iter().filter_map(|input| match input {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat_ty) => Some(&pat_ty.ty),
    });
    let output_type = match &item.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(ty),
    };

    let type_assertions: BTreeMap<_, _> = input_types
        .chain(output_type)
        .filter_map(|ty| {
            metadata::convert_type(ty).ok().map(|meta_ty| {
                let expected_ty = convert_type_back(&meta_ty);
                let assert = quote! {
                    ::uniffi::deps::static_assertions::assert_type_eq_all!(#ty, #expected_ty);
                };
                (meta_ty, assert)
            })
        })
        .collect();

    type_assertions.into_values().collect()
}

fn create_metadata_static_var(name: &Ident, val: Metadata) -> TokenStream {
    let data: Vec<u8> = bincode::serialize(&val).expect("Error serializing metadata item");
    let count = data.len();
    let var_name = format_ident!("UNIFFI_META_{}", name);

    quote! {
        #[no_mangle]
        pub static #var_name: [u8; #count] = [#(#data),*];
    }
}
