/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, Ident, ItemTrait, TraitItem, TraitItemFn};
use uniffi_meta::{CallbackInterfaceMetadata, ObjectMetadata};

use crate::{
    attrs::{MethodAttributes, TraitAttributes, TraitExportType},
    paths::LookupCache,
    Argument, Ir, RPath, Result, ReturnType, SelfArg,
};

pub struct Trait {
    pub attrs: TraitAttributes,
    pub ident: Ident,
    pub methods: Vec<TraitMethod>,
}

#[derive(Clone)]
pub struct TraitMethod {
    pub attrs: MethodAttributes,
    pub is_async: bool,
    pub ident: Ident,
    pub self_arg: SelfArg,
    pub args: Vec<Argument>,
    pub return_type: ReturnType,
}

impl Trait {
    pub fn parse(attrs: TraitAttributes, tr: ItemTrait) -> syn::Result<Self> {
        let mut methods = vec![];
        for item in tr.items {
            if let TraitItem::Fn(f) = item {
                if let Some(attrs) = MethodAttributes::parse(&f.attrs)? {
                    methods.push(TraitMethod::parse(attrs, f)?);
                }
            }
        }

        Ok(Self {
            attrs,
            ident: tr.ident,
            methods,
        })
    }

    pub fn trait_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<Vec<uniffi_meta::Metadata>> {
        let mut items = vec![];

        let trait_name = self.ident.unraw().to_string();
        let (trait_name, orig_name) = match &self.attrs.name {
            None => (trait_name, None),
            Some(name) => (name.clone(), Some(trait_name)),
        };
        let self_ty = match self.attrs.export_ty {
            TraitExportType::TraitInterface => {
                items.push(
                    ObjectMetadata {
                        module_path: module_path.path_string(),
                        name: trait_name.clone(),
                        orig_name,
                        docstring: self.attrs.docstring.clone(),
                        imp: uniffi_meta::ObjectImpl::Trait,
                        remote: false,
                    }
                    .into(),
                );
                uniffi_meta::Type::Object {
                    module_path: module_path.path_string(),
                    name: trait_name.clone(),
                    imp: uniffi_meta::ObjectImpl::Trait,
                }
            }
            TraitExportType::TraitInterfaceWithForeign => {
                items.push(
                    ObjectMetadata {
                        module_path: module_path.path_string(),
                        name: trait_name.clone(),
                        orig_name,
                        docstring: self.attrs.docstring.clone(),
                        imp: uniffi_meta::ObjectImpl::CallbackTrait,
                        remote: false,
                    }
                    .into(),
                );
                uniffi_meta::Type::Object {
                    module_path: module_path.path_string(),
                    name: trait_name.clone(),
                    imp: uniffi_meta::ObjectImpl::CallbackTrait,
                }
            }
            TraitExportType::CallbackInterface => {
                items.push(
                    CallbackInterfaceMetadata {
                        module_path: module_path.path_string(),
                        name: trait_name.clone(),
                        docstring: self.attrs.docstring.clone(),
                    }
                    .into(),
                );
                uniffi_meta::Type::CallbackInterface {
                    module_path: module_path.path_string(),
                    name: trait_name.clone(),
                }
            }
        };
        for (i, m) in self.methods.iter().enumerate() {
            items.push(
                m.to_trait_method_metadata(ir, cache, module_path, &trait_name, &self_ty, i)?
                    .into(),
            );
        }
        Ok(items)
    }
}

impl TraitMethod {
    pub fn parse(attrs: MethodAttributes, f: TraitItemFn) -> syn::Result<Self> {
        let mut inputs = f.sig.inputs.into_iter();
        let self_arg = SelfArg::parse(inputs.next(), f.sig.ident.span())?;

        Ok(Self {
            attrs,
            ident: f.sig.ident,
            is_async: f.sig.asyncness.is_some(),
            self_arg,
            args: inputs
                .map(Argument::parse)
                .collect::<syn::Result<Vec<_>>>()?,
            return_type: ReturnType::parse(f.sig.output)?,
        })
    }

    pub fn to_trait_method_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        trait_name: &str,
        self_ty: &uniffi_meta::Type,
        index: usize,
    ) -> Result<uniffi_meta::TraitMethodMetadata> {
        let (return_type, throws) =
            self.return_type
                .return_type_and_throws_for_method(ir, cache, module_path, self_ty)?;
        let item_name = self.ident.unraw().to_string();
        let (name, orig_name) = match &self.attrs.name {
            None => (item_name, None),
            Some(name) => (name.clone(), Some(item_name)),
        };

        Ok(uniffi_meta::TraitMethodMetadata {
            module_path: module_path.path_string(),
            trait_name: trait_name.to_string(),
            index: index as u32,
            name,
            orig_name,
            docstring: self.attrs.docstring.clone(),
            is_async: self.is_async,
            takes_self_by_arc: self.self_arg.takes_self_by_arc(ir, cache, module_path)?,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_method_metadata(ir, cache, module_path, self_ty))
                .collect::<Result<Vec<_>>>()?,
            return_type,
            throws,
            // Method checksums are not supported, we can implement an improved system by
            // checksumming the entire interface and having a single checksum
            checksum: None,
        })
    }
}
