/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::Attribute;

use crate::attrs::{extract_docstring, find_uniffi_derive};

#[derive(Clone, Default)]
pub struct RecordAttributes {
    pub docstring: Option<String>,
}

impl RecordAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();

        if !attrs
            .iter()
            .any(|a| find_uniffi_derive(&a.meta, "Record").is_some())
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

#[derive(Clone, Default)]
pub struct FieldAttributes {
    pub docstring: Option<String>,
}

impl FieldAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = FieldAttributes::default();
        for a in attrs {
            if a.meta.path().is_ident("doc") {
                extract_docstring(&mut parsed.docstring, &a.meta);
            }
        }
        Ok(Some(parsed))
    }
}
