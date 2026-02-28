/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Attribute, LitStr, Meta};

use crate::attrs::{extract_docstring, meta_matches_uniffi_export};

#[derive(Clone, Default)]
pub struct FunctionAttributes {
    pub docstring: Option<String>,
    pub async_runtime: Option<LitStr>,
}

impl FunctionAttributes {
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
                            parsed.async_runtime = meta.input.parse()?;
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
pub struct MethodAttributes {
    pub docstring: Option<String>,
    pub async_runtime: Option<LitStr>,
}

impl MethodAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();
        for a in attrs {
            if meta_matches_uniffi_export(&a.meta, "method") {
                if let Meta::List(list) = &a.meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("async_runtime") {
                            meta.value()?;
                            parsed.async_runtime = meta.input.parse()?;
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
        // Note: no need to check if we saw `uniffi::method`,
        // impl functions are methods unless otherwise flagged.
        Ok(Some(parsed))
    }
}

#[derive(Clone, Default)]
pub struct ConstructorAttributes {
    pub docstring: Option<String>,
    pub async_runtime: Option<LitStr>,
}

impl ConstructorAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();
        if !attrs
            .iter()
            .any(|a| meta_matches_uniffi_export(&a.meta, "constructor"))
        {
            return Ok(None);
        }
        for a in attrs {
            if meta_matches_uniffi_export(&a.meta, "constructor") {
                if let Meta::List(list) = &a.meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("async_runtime") {
                            meta.value()?;
                            parsed.async_runtime = meta.input.parse()?;
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
