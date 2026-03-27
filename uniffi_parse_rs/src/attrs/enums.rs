/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Attribute, LitStr, Meta};
use uniffi_meta::EnumShape;

use crate::{
    attrs::{extract_docstring, find_uniffi_derive},
    CompileEnv,
    ErrorKind::*,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumAttributes {
    pub shape: EnumShape,
    pub name: Option<String>,
    pub docstring: Option<String>,
    pub non_exhaustive: bool,
    pub discr_type: Option<uniffi_meta::Type>,
    pub remote: bool,
}

impl EnumAttributes {
    pub fn parse(env: &CompileEnv, attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut name = None;
        let mut docstring = None;
        let mut non_exhaustive = false;
        let mut discr_type = None;

        let Some(metas) = env.parse_attrs(attrs)? else {
            return Ok(None);
        };

        let enum_derive = metas
            .iter()
            .find_map(|meta| find_uniffi_derive(meta, "Enum"));
        let error_derive = metas
            .iter()
            .find_map(|meta| find_uniffi_derive(meta, "Error"));

        let (mut shape, remote) = match (enum_derive, error_derive) {
            (None, None) => return Ok(None),
            (Some(_), Some(d)) => return Err(syn::Error::new(d.span, MultipleEnumDerives)),
            (Some(d), None) => (EnumShape::Enum, d.remote),
            (None, Some(d)) => (EnumShape::Error { flat: false }, d.remote),
        };

        for meta in metas {
            let path = meta.path();
            if path.is_ident("uniffi") {
                if let Meta::List(list) = meta {
                    list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            meta.value()?;
                            let name_lit: LitStr = meta.input.parse()?;
                            name = Some(name_lit.value());
                            Ok(())
                        } else if meta.path.is_ident("flat_error") {
                            if let EnumShape::Error { flat } = &mut shape {
                                *flat = true;
                                Ok(())
                            } else {
                                Err(meta.error("flat_error invalid for non-error enumms"))
                            }
                        } else {
                            Err(meta.error("Invalid attribute"))
                        }
                    })?;
                }
            } else if meta.path().is_ident("doc") {
                extract_docstring(&mut docstring, &meta);
            } else if meta.path().is_ident("non_exhaustive") {
                non_exhaustive = true;
            } else if meta.path().is_ident("repr") {
                meta.require_list()?.parse_nested_meta(|meta| {
                    let Some(i) = meta.path.get_ident() else {
                        return Ok(());
                    };
                    if i == "u8" {
                        discr_type = Some(uniffi_meta::Type::UInt8);
                    } else if i == "i8" {
                        discr_type = Some(uniffi_meta::Type::Int8);
                    } else if i == "u16" {
                        discr_type = Some(uniffi_meta::Type::UInt16);
                    } else if i == "i16" {
                        discr_type = Some(uniffi_meta::Type::Int16);
                    } else if i == "u32" {
                        discr_type = Some(uniffi_meta::Type::UInt32);
                    } else if i == "i32" {
                        discr_type = Some(uniffi_meta::Type::Int32);
                    } else if i == "u64" {
                        discr_type = Some(uniffi_meta::Type::UInt64);
                    } else if i == "i64" {
                        discr_type = Some(uniffi_meta::Type::Int64);
                    } else {
                        return Err(meta.error("invalid repr"));
                    }
                    Ok(())
                })?;
            }
        }

        Ok(Some(EnumAttributes {
            shape,
            name,
            docstring,
            non_exhaustive,
            discr_type,
            remote,
        }))
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct VariantAttributes {
    pub name: Option<String>,
    pub docstring: Option<String>,
}

impl VariantAttributes {
    pub fn parse(env: &CompileEnv, attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut parsed = Self::default();
        let Some(metas) = env.parse_attrs(attrs)? else {
            return Ok(None);
        };
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
