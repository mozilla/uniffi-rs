/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use uniffi_meta::{checksum, FnMetadata, MethodMetadata, Type};

pub(crate) mod metadata;
mod scaffolding;

pub use self::metadata::gen_metadata;
use self::{
    metadata::convert::{convert_type, try_split_result},
    scaffolding::{gen_fn_scaffolding, gen_method_scaffolding},
};
use crate::util::{assert_type_eq, create_metadata_static_var};

// TODO(jplatte): Ensure no generics, no async, …
// TODO(jplatte): Aggregate errors instead of short-circuiting, wherever possible

pub enum ExportItem {
    Function {
        sig: Signature,
        metadata: FnMetadata,
    },
    Impl {
        self_ident: Ident,
        methods: Vec<syn::Result<Method>>,
    },
}

pub struct Method {
    sig: Signature,
    metadata: MethodMetadata,
}

pub struct Signature {
    ident: Ident,
    inputs: Vec<syn::FnArg>,
    output: Option<FunctionReturn>,
}

impl Signature {
    fn new(item: syn::Signature) -> syn::Result<Self> {
        let output = match item.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(FunctionReturn::new(ty)?),
        };

        Ok(Self {
            ident: item.ident,
            inputs: item.inputs.into_iter().collect(),
            output,
        })
    }
}

pub struct FunctionReturn {
    ty: Box<syn::Type>,
    throws: Option<Ident>,
}

impl FunctionReturn {
    fn new(ty: Box<syn::Type>) -> syn::Result<Self> {
        Ok(match try_split_result(&ty)? {
            Some((ok_type, throws)) => FunctionReturn {
                ty: Box::new(ok_type.to_owned()),
                throws: Some(throws),
            },
            None => FunctionReturn { ty, throws: None },
        })
    }
}

pub fn expand_export(metadata: ExportItem, mod_path: &[String]) -> TokenStream {
    match metadata {
        ExportItem::Function { sig, metadata } => {
            let checksum = checksum(&metadata);
            let scaffolding = gen_fn_scaffolding(&sig, mod_path, checksum);
            let type_assertions = fn_type_assertions(&sig);
            let meta_static_var = create_metadata_static_var(&sig.ident, metadata.into());

            quote! {
                #scaffolding
                #type_assertions
                #meta_static_var
            }
        }
        ExportItem::Impl {
            methods,
            self_ident,
        } => {
            let method_tokens: TokenStream = methods
                .into_iter()
                .map(|res| {
                    res.map_or_else(
                        syn::Error::into_compile_error,
                        |Method { sig, metadata }| {
                            let checksum = checksum(&metadata);
                            let scaffolding =
                                gen_method_scaffolding(&sig, mod_path, checksum, &self_ident);
                            let type_assertions = fn_type_assertions(&sig);
                            let meta_static_var = create_metadata_static_var(
                                &format_ident!("{}_{}", metadata.self_name, sig.ident),
                                metadata.into(),
                            );

                            quote! {
                                #scaffolding
                                #type_assertions
                                #meta_static_var
                            }
                        },
                    )
                })
                .collect();

            quote_spanned! {self_ident.span()=>
                ::uniffi::deps::static_assertions::assert_type_eq_all!(
                    #self_ident,
                    crate::uniffi_types::#self_ident
                );

                #method_tokens
            }
        }
    }
}

fn fn_type_assertions(sig: &Signature) -> TokenStream {
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
            Type::Unresolved { name } => {
                let ident = format_ident!("{name}");
                quote! { crate::uniffi_types::#ident }
            }
        }
    }

    let input_types = sig.inputs.iter().filter_map(|input| match input {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat_ty) => match &*pat_ty.pat {
            // Self type is asserted separately for impl blocks
            syn::Pat::Ident(i) if i.ident == "self" => None,
            _ => Some(&pat_ty.ty),
        },
    });
    let output_type = sig.output.as_ref().map(|s| &s.ty);

    let type_assertions: BTreeMap<_, _> = input_types
        .chain(output_type)
        .filter_map(|ty| {
            convert_type(ty).ok().map(|meta_ty| {
                let expected_ty = convert_type_back(&meta_ty);
                let assert = assert_type_eq(ty, expected_ty);
                (meta_ty, assert)
            })
        })
        .collect();
    let input_output_type_assertions: TokenStream = type_assertions.into_values().collect();

    let throws_type_assertion = sig.output.as_ref().and_then(|s| {
        let ident = s.throws.as_ref()?;
        Some(assert_type_eq(
            ident,
            quote! { crate::uniffi_types::#ident },
        ))
    });

    quote! {
        #input_output_type_assertions
        #throws_type_assertion
    }
}
