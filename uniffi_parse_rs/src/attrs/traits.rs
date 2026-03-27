/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{spanned::Spanned, Attribute, LitStr, Meta};

use uniffi_meta::TraitKind;

use crate::attrs::{extract_docstring, meta_matches_uniffi_export};

#[derive(Debug, PartialEq, Eq)]
pub struct TraitAttributes {
    pub name: Option<String>,
    pub export_ty: TraitExportType,
    pub docstring: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraitExportType {
    CallbackInterface,
    TraitInterface(TraitKind),
}

impl TraitExportType {
    pub fn is_trait(&self) -> bool {
        matches!(self, Self::TraitInterface(_))
    }

    pub fn is_callback_interface(&self) -> bool {
        matches!(self, Self::CallbackInterface)
    }
}

impl TraitAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut export_ty = None;
        let mut name = None;
        let mut docstring = None;

        for a in attrs {
            if meta_matches_uniffi_export(&a.meta, "export") {
                if export_ty.is_some() {
                    return Err(syn::Error::new(a.span(), "multiple `export` attributes"));
                }
                let mut saw_callback_interface = false;
                let mut saw_rust = false;
                let mut saw_foreign = false;
                if let Meta::List(list) = &a.meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            meta.value()?;
                            let lit: LitStr = meta.input.parse()?;
                            name = Some(lit.value());
                            Ok(())
                        } else if meta.path.is_ident("rust") {
                            saw_rust = true;
                            Ok(())
                        } else if meta.path.is_ident("foreign") {
                            saw_foreign = true;
                            Ok(())
                        } else if meta.path.is_ident("with_foreign") {
                            saw_rust = true;
                            saw_foreign = true;
                            Ok(())
                        } else if meta.path.is_ident("callback_interface") {
                            saw_callback_interface = true;
                            Ok(())
                        } else {
                            Err(meta.error("Invalid attribute"))
                        }
                    })?;
                }
                if (saw_rust || saw_foreign) && saw_callback_interface {
                    return Err(syn::Error::new(
                        a.span(),
                        "`callback_interface` not compatible with `rust`, `foreign` or `with_foreign`",
                    ));
                }
                export_ty = if saw_callback_interface {
                    Some(TraitExportType::CallbackInterface)
                } else {
                    let kind = match (saw_foreign, saw_rust) {
                        (true, true) => TraitKind::Both,
                        (true, false) => TraitKind::ForeignOnly,
                        _ => TraitKind::RustOnly,
                    };
                    Some(TraitExportType::TraitInterface(kind))
                }
            } else if a.meta.path().is_ident("doc") {
                extract_docstring(&mut docstring, &a.meta);
            }
        }
        if let Some(export_ty) = export_ty {
            Ok(Some(Self {
                export_ty,
                docstring,
                name,
            }))
        } else {
            Ok(None)
        }
    }
}
