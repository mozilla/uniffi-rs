/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Fields, Ident};

pub fn expand_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let type_name = &input.ident;
    let uniffi_meta_type_name = match parse_from_attr(&input.attrs)? {
        Some(name) => quote! { ::uniffi_meta::#name },
        None => quote! { ::uniffi_meta::#type_name },
    };
    let body = match input.data {
        Data::Struct(st) => {
            let (pattern, construct) = from_impl_patterns(st.fields)?;
            quote! {
                let #uniffi_meta_type_name #pattern = value;
                Self #construct
            }
        }
        Data::Enum(en) => {
            let cases = en.variants
                .into_iter()
                .map(|v| {
                    let (pattern, construct) = from_impl_patterns(v.fields)?;
                    let variant = &v.ident;
                    let uniffi_meta_variant = match parse_from_attr(&v.attrs)? {
                        Some(name) => quote! { #name },
                        None => quote! { #variant },
                    };
                    Ok(quote! {
                        #uniffi_meta_type_name::#uniffi_meta_variant #pattern => Self::#variant #construct,
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?;
            quote! {
                match value {
                    #(#cases)*
                }
            }
        }
        Data::Union(_) => return Err(syn::Error::new(type_name.span(), "unions not supported")),
    };
    Ok(quote! {
        impl From<#uniffi_meta_type_name> for #type_name {
            fn from(value: #uniffi_meta_type_name) -> Self {
                #body
            }
        }
    })
}

/// Generate patterns to destructure and construct a variant/struct
///
/// This is used to generate `From` implementations.  The patterns are used both for enum
/// matches and to destructure/reconstruct a struct.
fn from_impl_patterns(fields: Fields) -> syn::Result<(TokenStream, TokenStream)> {
    match fields {
        Fields::Unit => Ok((quote! {}, quote! {})),
        Fields::Named(fields) => {
            let mut source_fields = vec![];
            let mut dest_fields = vec![];
            for f in fields.named {
                source_fields.push(
                    parse_from_attr(&f.attrs)?.unwrap_or_else(|| f.ident.as_ref().unwrap().clone()),
                );
                dest_fields.push(f.ident);
            }
            Ok((
                quote! { { #(#source_fields,)* } },
                quote! { { #(#dest_fields: #source_fields.into()),* } },
            ))
        }
        Fields::Unnamed(fields) => {
            let var_names = (0..fields.unnamed.len()).map(|i| format_ident!("var{i}"));
            let var_names2 = var_names.clone();
            Ok((
                quote! { ( #(#var_names),* ) },
                quote! { ( #(#var_names2.into(),),* ) },
            ))
        }
    }
}

fn parse_from_attr(attrs: &[Attribute]) -> syn::Result<Option<Ident>> {
    for attr in attrs {
        if attr.path().is_ident("from") {
            return Ok(Some(attr.parse_args()?));
        }
    }
    Ok(None)
}
