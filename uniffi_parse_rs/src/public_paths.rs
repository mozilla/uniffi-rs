/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use crate::{paths::LookupCache, Error, ErrorKind::*, Ir, Item, Module, Namespace, RPath, Result};

/// Various names for an item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemNames {
    /// Path where we can import the module from another crate
    pub module_path: String,
    /// Name of the item, for the foreign bindings
    pub name: String,
    /// Original/Rust name of the item.
    ///
    /// The full import path for the item is `{module_path}::`orig_name}`
    /// Note that this means `orig_name` can be set even
    /// if there isn't a UniFFI attribute to rename the item.
    /// The other reason this is set is that the item lives in a non-public module and in the pub
    /// module where we can import it from, the `use` statement renames it.
    pub orig_name: Option<String>,
}

impl<'ir> RPath<'ir> {
    /// Find a public path for accessing the item at this path
    ///
    /// It returns:
    ///   * A module path string consisting of all public modules (e.g. "foo::bar::baz")
    ///   * The name of the item in the final module (which map be different than the canonical item,
    ///     if there are use statements with renames).
    pub fn public_path_to_item(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
    ) -> Result<ItemNames> {
        let cache_key = self.path_string();
        if let Some(result) = cache.public_paths.get(&cache_key) {
            return result.clone();
        }
        let result = self._public_path_to_item(ir, cache);
        cache.public_paths.insert(cache_key, result.clone());
        result
    }

    // Non-caching version of `public_path_to_item`
    fn _public_path_to_item(&self, ir: &'ir Ir, cache: &mut LookupCache<'ir>) -> Result<ItemNames> {
        trace!("Finding public path (module: {self})");
        let mut item = self.item()?;
        let mut module_path = self.parent()?;

        let namespace = match item {
            Item::Fn(_) => Namespace::Value,
            _ => Namespace::Type,
        };
        if let Item::CustomType(c) = item {
            let child = module_path.child(ir, cache, &c.ident, Namespace::NonUniffiType)?;
            item = child.path.item()?;
            module_path = child.path.parent()?;
            trace!("updated for custom type (module: {module_path} item: {item:?})");
        }

        // Track modules that we've searched.
        // This avoids infinite recursion when there are glob import cycles
        // and avoids doing extra work in general.
        let mut modules_searched = HashSet::new();
        let crate_root = module_path.crate_root()?;

        let Item::Module(module) = crate_root.item()? else {
            return Err(Error::internal(format!(
                "module_path_for_item: invalid module_path: {module_path}"
            )));
        };

        // Modules we're searching in the current iteration
        let mut modules_to_search = vec![(crate_root, module)];

        while !modules_to_search.is_empty() {
            let mut next_modules_to_search = vec![];
            // First, see if the item is a child of any of the current modules
            for (module_path, module) in modules_to_search.iter() {
                if let Some(result) = search_module(
                    ir,
                    cache,
                    module_path,
                    module,
                    item,
                    namespace,
                    &mut modules_searched,
                )? {
                    return Ok(result);
                }
                next_modules_to_search.extend(module.items.iter().filter_map(|i| match i {
                    Item::Module(m) if m.vis.is_pub() => Some((module_path.append_child(i), m)),
                    _ => None,
                }));
            }

            modules_to_search = next_modules_to_search;
        }

        Err(Error::new_without_location(NoPubPath(item.name())))
    }
}

fn search_module<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    module_path: &RPath<'ir>,
    module: &'ir Module,
    target: &Item,
    namespace: Namespace,
    modules_searched: &mut HashSet<usize>,
) -> Result<Option<ItemNames>> {
    trace!("searching: {module_path}");
    if !modules_searched.insert(module.id) {
        return Ok(None);
    }
    for item in module.items.iter() {
        if item.is(target) {
            let Some(item_ident) = target.ident() else {
                return Err(Error::internal(
                    "module_path_for_item: invalid item: {item:?}",
                ));
            };
            let item_names = match target.name_from_attrs() {
                None => ItemNames {
                    module_path: module_path.path_string(),
                    orig_name: None,
                    name: item_ident.to_string(),
                },
                Some(name) => ItemNames {
                    module_path: module_path.path_string(),
                    orig_name: Some(item_ident.to_string()),
                    name: name.to_string(),
                },
            };
            return Ok(Some(item_names));
        }
        if let Item::UseItem(use_item) = item {
            if use_item.vis.is_pub() {
                let resolved = match module_path.resolve(ir, cache, &use_item.path, namespace) {
                    Ok(r) => r,
                    Err(e) if e.is_not_found() => continue,
                    Err(e) => return Err(e),
                };
                if resolved.item()?.is(target) {
                    // `orig_name` is the name from the `use` statement, that way we can use
                    // module_path + orig_name to find the item in Rust code
                    let orig_name = use_item.ident.to_string();

                    // `name` the name from the item definition, this is what users expect to use
                    // in the foreign bindings.
                    let Some(item_ident) = target.ident() else {
                        return Err(Error::internal(
                            "module_path_for_item: invalid item: {item:?}",
                        ));
                    };
                    let name = target
                        .name_from_attrs()
                        .map(str::to_string)
                        .unwrap_or(item_ident.to_string());

                    let item_names = if name == orig_name {
                        ItemNames {
                            module_path: module_path.path_string(),
                            orig_name: None,
                            name,
                        }
                    } else {
                        ItemNames {
                            module_path: module_path.path_string(),
                            orig_name: Some(orig_name),
                            name,
                        }
                    };
                    return Ok(Some(item_names));
                }
            }
        }
        if let Item::UseGlob(use_glob) = item {
            if use_glob.vis.is_pub() {
                let resolved =
                    match module_path.resolve(ir, cache, &use_glob.module_path, namespace) {
                        Ok(r) => r,
                        Err(e) if e.is_not_found() => continue,
                        Err(e) => return Err(e),
                    };

                let Item::Module(module) = resolved.item()? else {
                    return Err(Error::internal(format!(
                        "module_path_for_item: use_glob.module_path is not a module ({module_path}, {:?})",
                        use_glob.module_path,
                    )));
                };

                if let Some(item_names) = search_module(
                    ir,
                    cache,
                    &resolved,
                    module,
                    target,
                    namespace,
                    modules_searched,
                )? {
                    return Ok(Some(ItemNames {
                        // Replace module_path with our module path, since we have a use glob
                        // pointing to that module.
                        module_path: module_path.to_string(),
                        ..item_names
                    }));
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use crate::paths::tests::path_for_module;

    use super::*;

    pub fn run_public_path_to_item(ir: &Ir, item_path: &str) -> ItemNames {
        let mut cache = LookupCache::default();
        let crate_path = path_for_module(ir, item_path.split("::").next().unwrap());
        let item = crate_path
            .resolve(
                ir,
                &mut cache,
                &syn::parse_str(item_path).unwrap(),
                Namespace::Type,
            )
            .unwrap();
        item.public_path_to_item(ir, &mut cache)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn test_find_public_paths() {
        let ir = Ir::new_for_test(&["public_paths"]);

        assert_eq!(
            run_public_path_to_item(&ir, "public_paths::Rec1"),
            ItemNames {
                module_path: "public_paths".into(),
                name: "Rec1".into(),
                orig_name: None,
            }
        );
        assert_eq!(
            run_public_path_to_item(&ir, "public_paths::mod1::mod2::Rec2"),
            ItemNames {
                module_path: "public_paths::mod1::mod2".into(),
                name: "Rec2".into(),
                orig_name: None,
            }
        );
        assert_eq!(
            run_public_path_to_item(&ir, "public_paths::nonpub::Rec3"),
            ItemNames {
                module_path: "public_paths::mod1".into(),
                name: "Rec3".into(),
                orig_name: None,
            }
        );
        assert_eq!(
            run_public_path_to_item(&ir, "public_paths::nonpub::Rec4"),
            ItemNames {
                module_path: "public_paths".into(),
                name: "Rec4".into(),
                orig_name: Some("RenamedRec4".into()),
            }
        );
        assert_eq!(
            run_public_path_to_item(&ir, "public_paths::nonpub2::Rec5"),
            ItemNames {
                module_path: "public_paths::mod5".into(),
                name: "Rec5".into(),
                orig_name: None,
            }
        );

        assert_eq!(
            run_public_path_to_item(&ir, "public_paths::nonpub::CustomType"),
            ItemNames {
                module_path: "public_paths::mod1".into(),
                name: "CustomType".into(),
                orig_name: None,
            }
        );
    }
}
