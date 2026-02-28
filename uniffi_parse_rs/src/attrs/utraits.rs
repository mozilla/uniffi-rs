/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{spanned::Spanned, Meta};

use crate::{attrs::meta_matches_uniffi_export, Error, RPath, Result};

#[derive(Clone, Default, Debug)]
pub struct UniffiTraitAttrs {
    debug: bool,
    display: bool,
    eq: bool,
    hash: bool,
    ord: bool,
}

impl UniffiTraitAttrs {
    pub fn parse(metas: &[Meta]) -> syn::Result<Self> {
        let mut parsed = Self::default();
        for meta in metas {
            if meta_matches_uniffi_export(meta, "export") {
                let Meta::List(list) = meta else {
                    return Err(syn::Error::new(meta.span(), "invalid attribute"));
                };
                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("Debug") {
                        parsed.debug = true;
                    } else if meta.path.is_ident("Display") {
                        parsed.display = true;
                    } else if meta.path.is_ident("Eq") {
                        parsed.eq = true;
                    } else if meta.path.is_ident("Ord") {
                        parsed.ord = true;
                    } else if meta.path.is_ident("Hash") {
                        parsed.hash = true;
                    } else {
                        return Err(meta.error("Invalid attribute"));
                    }
                    Ok(())
                })?;
            }
        }
        Ok(parsed)
    }

    pub fn uniffi_trait_method_metadata(
        &self,
        module_path: &RPath,
        self_ty: uniffi_meta::Type,
    ) -> Result<Vec<uniffi_meta::UniffiTraitMetadata>> {
        let mut metadata = vec![];
        let Some(self_name) = self_ty.name() else {
            return Err(Error::internal(
                "uniffi_trait_method_metadata: invalid self type: {self_ty:?}",
            ));
        };
        let module_path = module_path.path_string();

        if self.debug {
            metadata.push(uniffi_meta::UniffiTraitMetadata::Debug {
                fmt: uniffi_meta::MethodMetadata {
                    module_path: module_path.clone(),
                    self_name: self_name.into(),
                    name: "uniffi_trait_debug".into(),
                    orig_name: None,
                    is_async: false,
                    inputs: vec![],
                    return_type: Some(uniffi_meta::Type::String),
                    throws: None,
                    takes_self_by_arc: false,
                    checksum: None,
                    docstring: None,
                },
            });
        }
        if self.display {
            metadata.push(uniffi_meta::UniffiTraitMetadata::Display {
                fmt: uniffi_meta::MethodMetadata {
                    module_path: module_path.clone(),
                    self_name: self_name.into(),
                    name: "uniffi_trait_display".into(),
                    orig_name: None,
                    is_async: false,
                    inputs: vec![],
                    return_type: Some(uniffi_meta::Type::String),
                    throws: None,
                    takes_self_by_arc: false,
                    checksum: None,
                    docstring: None,
                },
            });
        }
        if self.eq {
            metadata.push(uniffi_meta::UniffiTraitMetadata::Eq {
                eq: uniffi_meta::MethodMetadata {
                    module_path: module_path.clone(),
                    self_name: self_name.into(),
                    name: "uniffi_trait_eq_eq".into(),
                    orig_name: None,
                    is_async: false,
                    inputs: vec![uniffi_meta::FnParamMetadata {
                        name: "other".into(),
                        ty: self_ty.clone(),
                        by_ref: true,
                        optional: false,
                        default: None,
                    }],
                    return_type: Some(uniffi_meta::Type::Boolean),
                    throws: None,
                    takes_self_by_arc: false,
                    checksum: None,
                    docstring: None,
                },
                ne: uniffi_meta::MethodMetadata {
                    module_path: module_path.clone(),
                    self_name: self_name.into(),
                    name: "uniffi_trait_eq_ne".into(),
                    orig_name: None,
                    is_async: false,
                    inputs: vec![uniffi_meta::FnParamMetadata {
                        name: "other".into(),
                        ty: self_ty.clone(),
                        by_ref: true,
                        optional: false,
                        default: None,
                    }],
                    return_type: Some(uniffi_meta::Type::Boolean),
                    throws: None,
                    takes_self_by_arc: false,
                    checksum: None,
                    docstring: None,
                },
            });
        }
        if self.hash {
            metadata.push(uniffi_meta::UniffiTraitMetadata::Hash {
                hash: uniffi_meta::MethodMetadata {
                    module_path: module_path.clone(),
                    self_name: self_name.into(),
                    name: "uniffi_trait_hash".into(),
                    orig_name: None,
                    is_async: false,
                    inputs: vec![],
                    return_type: Some(uniffi_meta::Type::UInt64),
                    throws: None,
                    takes_self_by_arc: false,
                    checksum: None,
                    docstring: None,
                },
            });
        }
        if self.ord {
            metadata.push(uniffi_meta::UniffiTraitMetadata::Ord {
                cmp: uniffi_meta::MethodMetadata {
                    module_path: module_path.clone(),
                    self_name: self_name.into(),
                    name: "uniffi_trait_ord_cmp".into(),
                    orig_name: None,
                    is_async: false,
                    inputs: vec![uniffi_meta::FnParamMetadata {
                        name: "other".into(),
                        ty: self_ty.clone(),
                        by_ref: true,
                        optional: false,
                        default: None,
                    }],
                    return_type: Some(uniffi_meta::Type::Int8),
                    throws: None,
                    takes_self_by_arc: false,
                    checksum: None,
                    docstring: None,
                },
            });
        }

        Ok(metadata)
    }
}
