/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, Ident, ItemTrait, TraitItem, TraitItemFn};

use crate::{
    attrs::{MethodAttributes, TraitAttributes},
    Argument, Result, ReturnType,
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

    pub fn name(&self) -> String {
        self.ident.unraw().to_string()
    }

    pub fn trait_metadata(&self) -> Result<Vec<uniffi_meta::Metadata>> {
        let mut items = vec![];

        let trait_name = self.name();
        for (i, m) in self.methods.iter().enumerate() {
            items.push(m.to_trait_method_metadata(&trait_name, i)?.into());
        }
        Ok(items)
    }
}

impl TraitMethod {
    pub fn parse(attrs: MethodAttributes, f: TraitItemFn) -> syn::Result<Self> {
        let inputs = f.sig.inputs.into_iter().take(1);

        Ok(Self {
            attrs,
            ident: f.sig.ident,
            is_async: f.sig.asyncness.is_some(),
            args: inputs
                .map(Argument::parse)
                .collect::<syn::Result<Vec<_>>>()?,
            return_type: ReturnType::parse(f.sig.output)?,
        })
    }

    pub fn name(&self) -> String {
        self.ident.unraw().to_string()
    }

    pub fn to_trait_method_metadata<'ir>(
        &self,
        trait_name: &str,
        index: usize,
    ) -> Result<uniffi_meta::TraitMethodMetadata> {
        let (return_type, throws) = (None, None); // TODO

        Ok(uniffi_meta::TraitMethodMetadata {
            module_path: "".into(), // TODO
            trait_name: trait_name.to_string(),
            index: index as u32,
            name: self.name(),
            docstring: self.attrs.docstring.clone(),
            is_async: self.is_async,
            takes_self_by_arc: false,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_metadata())
                .collect::<Result<Vec<_>>>()?,
            return_type,
            throws,
            // Method checksums are not supported, we can implement an improved system by
            // checksumming the entire interface and having a single checksum
            checksum: None,
        })
    }
}
