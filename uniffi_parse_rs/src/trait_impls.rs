/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    paths::LookupCache, types, BuiltinItem, GenericArgs, Ir, Item, Namespace, RPath, Result,
};

/// Implementation of a known trait
#[derive(Debug)]
pub enum TraitImpl {
    From { ty: uniffi_meta::Type },
}

impl<'ir> RPath<'ir> {
    pub fn resolve_trait_impl(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        trait_impl: &syn::ItemImpl,
    ) -> Result<Option<TraitImpl>> {
        let Some((_, trait_path, _)) = &trait_impl.trait_ else {
            return Ok(None);
        };
        let trait_item = match self.resolve(ir, cache, trait_path, Namespace::Type) {
            Ok(rpath) => rpath.item()?,
            Err(_) => return Ok(None),
        };

        match trait_item {
            Item::Builtin(BuiltinItem::From) => {
                self.resolve_from_impl(ir, cache, trait_path, &trait_impl.self_ty)
            }
            _ => Ok(None),
        }
    }

    fn resolve_from_impl(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &syn::Path,
        self_type: &syn::Type,
    ) -> Result<Option<TraitImpl>> {
        let generics = GenericArgs::new(self.file_id(), path)?;
        let generic_ty = generics.resolve1(ir, cache, self)?;
        if generic_ty != types::Type::UnexpectedUniFFICallbackError {
            return Ok(None);
        }
        let ty = self.resolve_uniffi_meta_type(ir, cache, self_type, None)?;
        Ok(Some(TraitImpl::From { ty }))
    }
}
