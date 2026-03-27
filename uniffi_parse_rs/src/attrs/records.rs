/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Attribute, LitStr, Meta};

use crate::attrs::{extract_docstring, find_uniffi_derive, Default};

#[derive(Clone, Default)]
pub struct RecordAttributes {
    pub name: Option<String>,
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
            if a.meta.path().is_ident("uniffi") {
                if let Meta::List(list) = &a.meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            meta.value()?;
                            let name: LitStr = meta.input.parse()?;
                            parsed.name = Some(name.value());
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

#[derive(Clone, Default)]
pub struct FieldAttributes {
    pub name: Option<String>,
    pub default: Option<Default>,
    pub docstring: Option<String>,
}

impl FieldAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = FieldAttributes::default();
        for a in attrs {
            if a.meta.path().is_ident("uniffi") {
                if let Meta::List(list) = &a.meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            meta.value()?;
                            let name: LitStr = meta.input.parse()?;
                            parsed.name = Some(name.value());
                            Ok(())
                        } else if meta.path.is_ident("default") {
                            if parsed.default.is_some() {
                                return Err(meta.error("Multiple default values"));
                            }
                            parsed.default = Some(Default::parse(meta)?);
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
