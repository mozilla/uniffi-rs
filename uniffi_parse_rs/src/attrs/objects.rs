/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Attribute, LitStr, Meta};

use crate::{
    attrs::{extract_docstring, find_uniffi_derive, meta_matches_uniffi_export, UniffiTraitAttrs},
    CompileEnv,
};

#[derive(Clone, Default)]
pub struct ObjectAttributes {
    pub name: Option<String>,
    pub docstring: Option<String>,
    pub remote: bool,
    pub utraits: UniffiTraitAttrs,
}

impl ObjectAttributes {
    pub fn parse(env: &CompileEnv, attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();

        let Some(metas) = env.parse_attrs(attrs)? else {
            return Ok(None);
        };
        parsed.remote = match metas
            .iter()
            .find_map(|meta| find_uniffi_derive(meta, "Object"))
        {
            Some(d) => d.remote,
            None => return Ok(None),
        };

        parsed.utraits = UniffiTraitAttrs::parse(&metas)?;
        for meta in metas {
            if meta.path().is_ident("uniffi") {
                if let Meta::List(list) = meta {
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
            } else if meta.path().is_ident("doc") {
                extract_docstring(&mut parsed.docstring, &meta);
            }
        }
        Ok(Some(parsed))
    }
}

#[derive(Default)]
pub struct ImplAttributes {
    pub name: Option<String>,
    pub async_runtime: Option<LitStr>,
}

impl ImplAttributes {
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
                        if meta.path.is_ident("name") {
                            meta.value()?;
                            let name: LitStr = meta.input.parse()?;
                            parsed.name = Some(name.value());
                            Ok(())
                        } else if meta.path.is_ident("async_runtime") {
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
