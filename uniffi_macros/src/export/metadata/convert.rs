/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::Ident;
use quote::ToTokens;

fn type_as_type_name(arg: &syn::Type) -> syn::Result<&Ident> {
    type_as_type_path(arg)?
        .path
        .get_ident()
        .ok_or_else(|| type_not_supported(arg))
}

pub(super) fn type_as_type_path(ty: &syn::Type) -> syn::Result<&syn::TypePath> {
    match ty {
        syn::Type::Group(g) => type_as_type_path(&g.elem),
        syn::Type::Paren(p) => type_as_type_path(&p.elem),
        syn::Type::Path(p) => Ok(p),
        _ => Err(type_not_supported(ty)),
    }
}

fn arg_as_type(arg: &syn::GenericArgument) -> syn::Result<&syn::Type> {
    match arg {
        syn::GenericArgument::Type(t) => Ok(t),
        _ => Err(syn::Error::new_spanned(
            arg,
            "non-type generic parameters are not currently supported by uniffi::export",
        )),
    }
}

fn type_not_supported(ty: &impl ToTokens) -> syn::Error {
    syn::Error::new_spanned(
        ty,
        "this type is not currently supported by uniffi::export in this position",
    )
}

pub(crate) fn try_split_result(ty: &syn::Type) -> syn::Result<Option<(&syn::Type, Ident)>> {
    let type_path = type_as_type_path(ty)?;

    if type_path.qself.is_some() {
        return Err(syn::Error::new_spanned(
            type_path,
            "qualified self types are not currently supported by uniffi::export",
        ));
    }

    if type_path.path.segments.len() > 1 {
        return Err(syn::Error::new_spanned(
            type_path,
            "qualified paths in types are not currently supported by uniffi::export",
        ));
    }

    let (ident, a) = match &type_path.path.segments.first() {
        Some(seg) => match &seg.arguments {
            syn::PathArguments::AngleBracketed(a) => (&seg.ident, a),
            syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => return Ok(None),
        },
        None => return Ok(None),
    };

    let mut it = a.args.iter();
    if let Some(arg1) = it.next() {
        if let Some(arg2) = it.next() {
            if it.next().is_none() {
                let arg1 = arg_as_type(arg1)?;
                let arg2 = arg_as_type(arg2)?;

                if let "Result" = ident.to_string().as_str() {
                    let throws = type_as_type_name(arg2)?.to_owned();
                    return Ok(Some((arg1, throws)));
                }
            }
        }
    }

    Ok(None)
}
