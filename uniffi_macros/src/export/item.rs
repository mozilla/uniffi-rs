/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

pub(super) enum ExportItem {
    Function {
        sig: Signature,
    },
    Impl {
        self_ident: Ident,
        methods: Vec<syn::Result<Method>>,
    },
}

impl ExportItem {
    pub fn new(item: syn::Item) -> syn::Result<Self> {
        match item {
            syn::Item::Fn(item) => {
                let sig = Signature::new(item.sig)?;
                Ok(Self::Function { sig })
            }
            syn::Item::Impl(item) => Self::from_impl(item),
            // FIXME: Support const / static?
            _ => Err(syn::Error::new(
                Span::call_site(),
                "unsupported item: only functions and impl \
                 blocks may be annotated with this attribute",
            )),
        }
    }

    fn from_impl(item: syn::ItemImpl) -> syn::Result<Self> {
        if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
            return Err(syn::Error::new_spanned(
                &item.generics,
                "generic impls are not currently supported by uniffi::export",
            ));
        }

        let type_path = type_as_type_path(&item.self_ty)?;

        if type_path.qself.is_some() {
            return Err(syn::Error::new_spanned(
                type_path,
                "qualified self types are not currently supported by uniffi::export",
            ));
        }

        let self_ident = match type_path.path.get_ident() {
            Some(id) => id,
            None => {
                return Err(syn::Error::new_spanned(
                    type_path,
                    "qualified paths in self-types are not currently supported by uniffi::export",
                ));
            }
        };

        let methods = item.items.into_iter().map(Method::new).collect();

        Ok(Self::Impl {
            methods,
            self_ident: self_ident.to_owned(),
        })
    }
}

pub(super) struct Method {
    pub sig: Signature,
}

impl Method {
    fn new(it: syn::ImplItem) -> syn::Result<Self> {
        let sig = match it {
            syn::ImplItem::Method(m) => Signature::new(m.sig)?,
            _ => {
                return Err(syn::Error::new_spanned(
                    it,
                    "only methods are supported in impl blocks annotated with uniffi::export",
                ));
            }
        };

        Ok(Self { sig })
    }
}

pub(super) struct Signature {
    pub ident: Ident,
    pub is_async: bool,
    pub inputs: Vec<syn::FnArg>,
    pub output: TokenStream,
}

impl Signature {
    fn new(item: syn::Signature) -> syn::Result<Self> {
        let output = match item.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };

        Ok(Self {
            ident: item.ident,
            is_async: item.asyncness.is_some(),
            inputs: item.inputs.into_iter().collect(),
            output,
        })
    }
}

fn type_as_type_path(ty: &syn::Type) -> syn::Result<&syn::TypePath> {
    match ty {
        syn::Type::Group(g) => type_as_type_path(&g.elem),
        syn::Type::Paren(p) => type_as_type_path(&p.elem),
        syn::Type::Path(p) => Ok(p),
        _ => Err(type_not_supported(ty)),
    }
}

fn type_not_supported(ty: &impl ToTokens) -> syn::Error {
    syn::Error::new_spanned(
        ty,
        "this type is not currently supported by uniffi::export in this position",
    )
}
