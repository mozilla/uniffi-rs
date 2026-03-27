/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use syn::ItemMacro;

use crate::{paths::LookupCache, BuiltinItem, Error, Ir, Item, Module, RPath, Result};

/// Resolve Item::Macro to more specific items like Item::UseRemoteType
///
/// This needs to run after adding all crates parsing, since it requires looking up module paths.
pub fn resolve_macros(ir: &mut Ir) -> Result<()> {
    let mut cache = LookupCache::default();
    let mut found_items = HashMap::<usize, Vec<Item>>::new();
    for (mut path, module) in ir.crate_roots_and_paths() {
        resolve_macros_recurse(ir, &mut cache, module, &mut path, &mut found_items)?;
    }
    for module in ir.crate_roots_mut() {
        insert_found_items(module, &mut found_items);
    }

    Ok(())
}

fn resolve_macros_recurse<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    module: &'ir Module,
    path: &mut RPath<'ir>,
    found_items: &mut HashMap<usize, Vec<Item>>,
) -> Result<()> {
    for item in module.items.iter() {
        match item {
            Item::Macro(mac) => {
                if let Some(resolved) = maybe_resolve_macro(ir, cache, path, mac)? {
                    found_items.entry(module.id).or_default().push(resolved);
                }
            }
            Item::Module(child) => {
                path.push(item);
                resolve_macros_recurse(ir, cache, child, path, found_items)?;
                path.pop();
            }
            _ => (),
        }
    }
    Ok(())
}

/// Try resolving Item::Macro into a more specific item like Item::UseRemoteType
fn maybe_resolve_macro<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    path: &RPath<'ir>,
    mac: &ItemMacro,
) -> Result<Option<Item>> {
    let builtin = match path.resolve(ir, cache, &mac.mac.path) {
        // Ignore errors, maybe the macro comes from an unparsed crate.
        Err(_) => return Ok(None),
        Ok(path) => match path.item()? {
            Item::Builtin(b) => b,
            _ => return Ok(None),
        },
    };
    match builtin {
        BuiltinItem::UniffiMacro("use_remote_type") => Ok(Some(Item::UseRemoteType(
            mac.mac
                .parse_body()
                .map_err(|e| Error::new_syn(path.file_id(), e))?,
        ))),
        _ => Ok(None),
    }
}

fn insert_found_items(module: &mut Module, items: &mut HashMap<usize, Vec<Item>>) {
    if let Some(found_items) = items.remove(&module.id) {
        module.items.extend(found_items);
    }
    for item in module.items.iter_mut() {
        if let Item::Module(child) = item {
            insert_found_items(child, items);
        }
    }
}
