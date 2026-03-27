/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Attribute, LitStr, Meta};

use crate::attrs::{extract_docstring, meta_matches_uniffi_export};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct TraitAttributes {
    pub name: Option<String>,
    pub export_ty: TraitExportType,
    pub docstring: Option<String>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraitExportType {
    #[default]
    TraitInterface,
    TraitInterfaceWithForeign,
    CallbackInterface,
}

impl TraitExportType {
    pub fn is_trait(&self) -> bool {
        matches!(self, Self::TraitInterface | Self::TraitInterfaceWithForeign)
    }

    pub fn is_callback_interface(&self) -> bool {
        matches!(self, Self::CallbackInterface)
    }
}

impl TraitAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();
        if !attrs
            .iter()
            .any(|a| meta_matches_uniffi_export(&a.meta, "export"))
        {
            return Ok(None);
        }

        for a in attrs {
            if meta_matches_uniffi_export(&a.meta, "export") {
                if let Meta::List(list) = &a.meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            meta.value()?;
                            let name: LitStr = meta.input.parse()?;
                            parsed.name = Some(name.value());
                            Ok(())
                        } else if meta.path.is_ident("with_foreign") {
                            parsed.export_ty = TraitExportType::TraitInterfaceWithForeign;
                            Ok(())
                        } else if meta.path.is_ident("callback_interface") {
                            parsed.export_ty = TraitExportType::CallbackInterface;
                            Ok(())
                        } else {
                            Err(meta.error("Invalid attribute"))
                        }
                    })?;
                }
            } else if a.meta.path().is_ident("doc") {
                extract_docstring(&mut parsed.docstring, &a.meta);
            }
        }
        Ok(Some(parsed))
    }
}
