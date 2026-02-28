/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Attribute, LitStr, Meta};

use crate::attrs::{extract_docstring, find_uniffi_derive, meta_matches_uniffi_export};

#[derive(Clone, Default)]
pub struct ObjectAttributes {
    pub docstring: Option<String>,
}

impl ObjectAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();

        if !attrs
            .iter()
            .any(|a| find_uniffi_derive(&a.meta, "Object").is_some())
        {
            return Ok(None);
        }

        for a in attrs {
            if a.meta.path().is_ident("doc") {
                extract_docstring(&mut parsed.docstring, &a.meta);
            }
        }
        Ok(Some(parsed))
    }
}

#[derive(Default)]
pub struct ImplAttributes {
    pub async_runtime: Option<LitStr>,
}

impl ImplAttributes {
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
                        if meta.path.is_ident("async_runtime") {
                            meta.value()?;
                            parsed.async_runtime = Some(meta.input.parse()?);
                            Ok(())
                        } else {
                            Err(meta.error("Invalid attribute"))
                        }
                    })?;
                }
            }
        }
        Ok(Some(parsed))
    }
}
