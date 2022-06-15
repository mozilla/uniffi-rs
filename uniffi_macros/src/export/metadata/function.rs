/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::Span;
use uniffi_meta::{checksum, FnMetadata, FnParamMetadata};

use super::{convert_type, write_json_metadata, METADATA_DIR};
use crate::export::ExportItem;

pub(super) fn gen_fn_metadata(item: syn::ItemFn, mod_path: &[String]) -> syn::Result<ExportItem> {
    let meta = fn_metadata(&item, mod_path)?;

    let path = METADATA_DIR.join(format!(
        "mod.{}.fn.{}.json",
        // `-` is a character that's practically universally allowed in
        // filenames, yet can't be part of a module path segment.
        meta.module_path.join("-"),
        meta.name
    ));

    let tracked_file = write_json_metadata(&path, &meta)
        .map_err(|e| syn::Error::new(Span::call_site(), format!("failed to write file: {}", e)))?;

    Ok(ExportItem::Function {
        item,
        checksum: checksum(&meta),
        tracked_file,
    })
}

fn fn_metadata(f: &syn::ItemFn, mod_path: &[String]) -> syn::Result<FnMetadata> {
    let output = match &f.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(convert_type(ty)?),
    };

    Ok(FnMetadata {
        module_path: mod_path.to_owned(),
        name: f.sig.ident.to_string(),
        inputs: f
            .sig
            .inputs
            .iter()
            .map(|a| fn_param_metadata(a, false))
            .collect::<syn::Result<_>>()?,
        output,
    })
}

fn fn_param_metadata(a: &syn::FnArg, _is_method: bool) -> syn::Result<FnParamMetadata> {
    let (name, ty) = match a {
        syn::FnArg::Receiver(_) => unimplemented!(),
        syn::FnArg::Typed(pat_ty) => {
            let name = match &*pat_ty.pat {
                syn::Pat::Ident(pat_id) => pat_id.ident.to_string(),
                _ => unimplemented!(),
            };
            (name, &pat_ty.ty)
        }
    };

    Ok(FnParamMetadata {
        name,
        ty: convert_type(ty)?,
    })
}
