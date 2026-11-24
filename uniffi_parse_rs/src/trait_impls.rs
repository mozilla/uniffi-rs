/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    paths::LookupCache, types, BuiltinItem, GenericArgs, Ir, IrChanges, Item, Namespace, RPath,
    Result, VisitUnresolvedItem,
};

pub fn resolve_trait_items(ir: &mut Ir) -> Result<()> {
    let mut changes = IrChanges::default();
    let mut cache = LookupCache::default();
    ir.try_visit_unresolved_items(|_, module, path, unresolved| {
        let syn::Item::Impl(trait_impl) = unresolved else {
            return Ok(VisitUnresolvedItem::Keep);
        };

        let Some((_, trait_path, _)) = &trait_impl.trait_ else {
            return Ok(VisitUnresolvedItem::Keep);
        };
        let trait_item = match path.resolve(ir, &mut cache, trait_path, Namespace::Type) {
            Ok(rpath) => rpath.item()?,
            Err(_) => return Ok(VisitUnresolvedItem::Keep),
        };

        match trait_item {
            Item::Builtin(BuiltinItem::From) => {
                match resolve_from_impl(ir, &mut cache, path, trait_path, &trait_impl.self_ty) {
                    Ok(Some(i)) => {
                        changes.add(module.id, i);
                        Ok(VisitUnresolvedItem::Remove)
                    }
                    _ => Ok(VisitUnresolvedItem::Keep),
                }
            }
            _ => Ok(VisitUnresolvedItem::Keep),
        }
    })?;
    changes.apply(ir)?;
    Ok(())
}

fn resolve_from_impl<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    module_path: &RPath<'ir>,
    path: &syn::Path,
    self_type: &syn::Type,
) -> Result<Option<Item>> {
    let generics = GenericArgs::new(module_path.file_id(), path)?;
    let generic_ty = generics.resolve1(ir, cache, module_path)?;
    if generic_ty != types::Type::UnexpectedUniFFICallbackError {
        return Ok(None);
    }
    let ty = module_path.resolve_uniffi_meta_type(ir, cache, self_type, None)?;
    Ok(Some(Item::FromUnexpectedUniffiCallbackError(ty)))
}
