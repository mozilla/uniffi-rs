/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::convert::type_as_type_path;
use crate::export::{ExportItem, Method, Signature};

pub(super) fn gen_impl_metadata(item: syn::ItemImpl) -> syn::Result<ExportItem> {
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

    let methods = item.items.into_iter().map(gen_method_metadata).collect();

    Ok(ExportItem::Impl {
        methods,
        self_ident: self_ident.to_owned(),
    })
}

fn gen_method_metadata(it: syn::ImplItem) -> syn::Result<Method> {
    let sig = match it {
        syn::ImplItem::Method(m) => Signature::new(m.sig)?,
        _ => {
            return Err(syn::Error::new_spanned(
                it,
                "only methods are supported in impl blocks annotated with uniffi::export",
            ));
        }
    };

    Ok(Method { sig })
}
