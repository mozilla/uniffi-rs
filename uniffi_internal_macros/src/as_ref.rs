/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Field, Fields, Member, Result};

pub fn expand_as_ref(input: DeriveInput) -> Result<TokenStream> {
    let type_name = &input.ident;
    let st = match input.data {
        Data::Struct(st) => st,
        Data::Enum(_) => return Err(syn::Error::new(input.ident.span(), "Enums not supported")),
        Data::Union(_) => return Err(syn::Error::new(input.ident.span(), "Union not supported")),
    };
    let mut tokens = TokenStream::default();

    if has_as_ref_attr(&input.attrs) {
        tokens.extend(quote! {
            impl ::std::convert::AsRef<#type_name> for #type_name {
                fn as_ref(&self) -> &#type_name {
                    self
                }
            }
        });
    }

    for (m, f) in members_and_fields(&st.fields) {
        if has_as_ref_attr(&f.attrs) {
            let ty = &f.ty;
            tokens.extend(quote! {
                impl ::std::convert::AsRef<#ty> for #type_name {
                    fn as_ref(&self) -> &#ty {
                        &self.#m
                    }
                }

            })
        }
    }
    Ok(tokens)
}

fn members_and_fields(fields: &Fields) -> impl Iterator<Item = (Member, &Field)> {
    fields.iter().enumerate().map(|(idx, f)| {
        let member = match &f.ident {
            Some(ident) => Member::from(ident.clone()),
            None => Member::from(idx),
        };
        (member, f)
    })
}

fn has_as_ref_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("as_ref"))
}
