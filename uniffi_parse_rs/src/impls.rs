/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{spanned::Spanned, ImplItem, ItemImpl};

use crate::{
    attrs::{ConstructorAttributes, ImplAttributes, MethodAttributes},
    paths::LookupCache,
    CompileEnv, Constructor, Error,
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
    pub fn parse(env: &CompileEnv, attrs: ImplAttributes, imp: ItemImpl) -> syn::Result<Self> {
        let mut constructors = vec![];
        let mut methods = vec![];
        for item in imp.items {
            if let ImplItem::Fn(f) = item {
                // impl-level `cancellable` applies to every async fn in the block.  Sync fns are
                // skipped rather than rejected, so a block can mix the two.
                let inherits_cancellable = attrs.cancellable && f.sig.asyncness.is_some();
                if let Some(mut a) = ConstructorAttributes::parse(env, &f.attrs)? {
                    a.cancellable |= inherits_cancellable;
                    constructors.push(Constructor::parse(a, f)?);
                } else if let Some(mut a) = MethodAttributes::parse(env, &f.attrs)? {
                    a.cancellable |= inherits_cancellable;
                    methods.push(Method::parse(a, f)?);
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
        let self_name = match (&self.attrs.name, self_ty.name()) {
            (Some(n), _) => n.to_string(),
            (None, Some(n)) => n.to_string(),
            (None, None) => {
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
