/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use super::attributes::ExportedImplFnAttributes;

pub(super) enum ExportItem {
    Function {
        sig: FnSignature,
    },
    Impl {
        self_ident: Ident,
        items: Vec<syn::Result<ImplItem>>,
    },
    Trait {
        self_ident: Ident,
        items: Vec<syn::Result<ImplItem>>,
    },
}

impl ExportItem {
    pub fn new(item: syn::Item) -> syn::Result<Self> {
        match item {
            syn::Item::Fn(item) => {
                let sig = FnSignature::new(item.sig)?;
                Ok(Self::Function { sig })
            }
            syn::Item::Impl(item) => Self::from_impl(item),
            syn::Item::Trait(item) => Self::from_trait(item),
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

        let items = item
            .items
            .into_iter()
            .map(|item| {
                let impl_fn = match item {
                    syn::ImplItem::Method(m) => m,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            item,
                            "only fn's are supported in impl blocks annotated with uniffi::export",
                        ));
                    }
                };

                let attrs = ExportedImplFnAttributes::new(&impl_fn.attrs)?;
                let item = if attrs.constructor {
                    ImplItem::Constructor(ConstructorSignature::new(impl_fn.sig)?)
                } else {
                    ImplItem::Method(FnSignature::new(impl_fn.sig)?)
                };

                Ok(item)
            })
            .collect();

        Ok(Self::Impl {
            items,
            self_ident: self_ident.to_owned(),
        })
    }

    fn from_trait(item: syn::ItemTrait) -> syn::Result<Self> {
        if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
            return Err(syn::Error::new_spanned(
                &item.generics,
                "generic impls are not currently supported by uniffi::export",
            ));
        }

        let self_ident = item.ident.to_owned();
        let items = item
            .items
            .into_iter()
            .map(|item| {
                let tim = match item {
                    syn::TraitItem::Method(tim) => tim,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            item,
                            "only fn's are supported in traits annotated with uniffi::export",
                        ));
                    }
                };

                let attrs = ExportedImplFnAttributes::new(&tim.attrs)?;
                let item = if attrs.constructor {
                    return Err(syn::Error::new_spanned(
                        tim,
                        "traits can not have constructors",
                    ));
                } else {
                    ImplItem::Method(FnSignature::new(tim.sig)?)
                };

                Ok(item)
            })
            .collect();

        Ok(Self::Trait { items, self_ident })
    }
}

pub(super) enum ImplItem {
    Constructor(ConstructorSignature),
    Method(FnSignature),
}

pub(super) struct FnSignature {
    pub ident: Ident,
    pub is_async: bool,
    pub inputs: Vec<syn::FnArg>,
    pub output: TokenStream,
}

impl FnSignature {
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

pub(super) struct ConstructorSignature {
    pub ident: Ident,
    pub inputs: Vec<syn::FnArg>,
    pub output: TokenStream,
}

impl ConstructorSignature {
    fn new(item: syn::Signature) -> syn::Result<Self> {
        let output = match item.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };

        Ok(Self {
            ident: item.ident,
            inputs: item.inputs.into_iter().collect(),
            output,
        })
    }
}

impl From<ConstructorSignature> for FnSignature {
    fn from(value: ConstructorSignature) -> Self {
        Self {
            ident: value.ident,
            is_async: false,
            inputs: value.inputs,
            output: value.output,
        }
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
