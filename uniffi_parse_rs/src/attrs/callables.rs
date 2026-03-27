/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Attribute, LitStr, Meta};

use crate::{
    attrs::{extract_docstring, meta_matches_uniffi_export, DefaultMap},
    CompileEnv,
};

#[derive(Clone, Default)]
pub struct FunctionAttributes {
    pub defaults: DefaultMap,
    pub name: Option<String>,
    pub docstring: Option<String>,
    pub async_runtime: Option<LitStr>,
}

impl FunctionAttributes {
    pub fn parse(env: &CompileEnv, attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();
        let Some(metas) = env.parse_attrs(attrs)? else {
            return Ok(None);
        };
        if !metas
            .iter()
            .any(|meta| meta_matches_uniffi_export(meta, "export"))
        {
            return Ok(None);
        }
        for meta in metas {
            if meta_matches_uniffi_export(&meta, "export") {
                if let Meta::List(list) = meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("default") {
                            parsed.defaults.parse(meta)
                        } else if meta.path.is_ident("name") {
                            meta.value()?;
                            let name: LitStr = meta.input.parse()?;
                            parsed.name = Some(name.value());
                            Ok(())
                        } else if meta.path.is_ident("async_runtime") {
                            meta.value()?;
                            parsed.async_runtime = meta.input.parse()?;
                            Ok(())
                        } else {
                            Err(meta.error("Invalid attribute"))
                        }
                    })?;
                }
            } else if meta.path().is_ident("doc") {
                extract_docstring(&mut parsed.docstring, &meta);
            }
        }
        Ok(Some(parsed))
    }
}

#[derive(Clone, Default)]
pub struct MethodAttributes {
    pub defaults: DefaultMap,
    pub name: Option<String>,
    pub docstring: Option<String>,
    pub async_runtime: Option<LitStr>,
}

impl MethodAttributes {
    pub fn parse(env: &CompileEnv, attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();
        let Some(metas) = env.parse_attrs(attrs)? else {
            return Ok(None);
        };
        for meta in metas {
            if meta_matches_uniffi_export(&meta, "method") {
                if let Meta::List(list) = meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("default") {
                            parsed.defaults.parse(meta)
                        } else if meta.path.is_ident("name") {
                            meta.value()?;
                            let name: LitStr = meta.input.parse()?;
                            parsed.name = Some(name.value());
                            Ok(())
                        } else if meta.path.is_ident("async_runtime") {
                            meta.value()?;
                            parsed.async_runtime = meta.input.parse()?;
                            Ok(())
                        } else {
                            Err(meta.error("Invalid attribute"))
                        }
                    })?;
                }
            } else if meta.path().is_ident("doc") {
                extract_docstring(&mut parsed.docstring, &meta);
            }
        }
        // Note: no need to check if we saw `uniffi::method`,
        // impl functions are methods unless otherwise flagged.
        Ok(Some(parsed))
    }
}

#[derive(Clone, Default)]
pub struct ConstructorAttributes {
    pub defaults: DefaultMap,
    pub name: Option<String>,
    pub docstring: Option<String>,
    pub async_runtime: Option<LitStr>,
}

impl ConstructorAttributes {
    pub fn parse(env: &CompileEnv, attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();
        let Some(metas) = env.parse_attrs(attrs)? else {
            return Ok(None);
        };
        if !metas
            .iter()
            .any(|meta| meta_matches_uniffi_export(meta, "constructor"))
        {
            return Ok(None);
        }
        for meta in metas {
            if meta_matches_uniffi_export(&meta, "constructor") {
                if let Meta::List(list) = meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("default") {
                            parsed.defaults.parse(meta)
                        } else if meta.path.is_ident("name") {
                            meta.value()?;
                            let name: LitStr = meta.input.parse()?;
                            parsed.name = Some(name.value());
                            Ok(())
                        } else if meta.path.is_ident("async_runtime") {
                            meta.value()?;
                            parsed.async_runtime = meta.input.parse()?;
                            Ok(())
                        } else {
                            Err(meta.error("Invalid attribute"))
                        }
                    })?;
                }
            } else if meta.path().is_ident("doc") {
                extract_docstring(&mut parsed.docstring, &meta);
            }
        }
        Ok(Some(parsed))
    }
}
