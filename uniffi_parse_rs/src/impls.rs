/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{spanned::Spanned, ImplItem, ItemImpl};

use crate::{
    attrs::{ConstructorAttributes, ImplAttributes, MethodAttributes},
    paths::LookupCache,
    Constructor, Error,
    ErrorKind::*,
    Ir, Method, RPath, Result,
};

pub struct Impl {
    pub attrs: ImplAttributes,
    pub self_type: syn::Type,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
}

impl Impl {
    pub fn parse(attrs: ImplAttributes, imp: ItemImpl) -> syn::Result<Self> {
        let mut constructors = vec![];
        let mut methods = vec![];
        for item in imp.items {
            if let ImplItem::Fn(f) = item {
                if let Some(attrs) = ConstructorAttributes::parse(&f.attrs)? {
                    constructors.push(Constructor::parse(attrs, f)?);
                } else if let Some(attrs) = MethodAttributes::parse(&f.attrs)? {
                    methods.push(Method::parse(attrs, f)?);
                }
            }
        }

        Ok(Self {
            attrs,
            self_type: *imp.self_ty,
            constructors,
            methods,
        })
    }

    pub fn impl_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<Vec<uniffi_meta::Metadata>> {
        let self_ty = module_path.resolve_uniffi_meta_type(ir, cache, &self.self_type, None)?;
        let self_name = match self_ty.name() {
            Some(n) => n.to_string(),
            None => {
                return Err(Error::new(
                    module_path.file_id(),
                    self.self_type.span(),
                    InvalidSelfType,
                ))
            }
        };
        let mut items = vec![];
        for c in self.constructors.iter() {
            items.push(
                c.to_constructor_metadata(ir, cache, module_path, &self_name, &self_ty)?
                    .into(),
            );
        }
        for m in self.methods.iter() {
            items.push(
                m.to_method_metadata(ir, cache, module_path, &self_name, &self_ty)?
                    .into(),
            );
        }
        Ok(items)
    }
}
