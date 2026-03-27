/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{punctuated::Punctuated, Ident, ItemUse, Token, UseTree};

use crate::{ErrorKind::*, Item};

pub struct UseItem {
    // Path to the item being imported
    pub path: syn::Path,
    // Name of the import
    pub ident: Ident,
}

pub struct UseGlob {
    // Path to the module that's being imported
    pub module_path: syn::Path,
    pub star_token: Token![*],
}

pub fn parse_use(use_item: ItemUse) -> syn::Result<Vec<Item>> {
    let path = syn::Path {
        leading_colon: use_item.leading_colon,
        segments: Punctuated::new(),
    };
    let mut items = vec![];
    parse_use_recurse(&path, use_item.tree, &mut items)?;
    Ok(items)
}

fn parse_use_recurse(
    current_path: &syn::Path,
    mut tree: UseTree,
    items: &mut Vec<Item>,
) -> syn::Result<()> {
    let mut current_path = current_path.clone();

    // Process `UseTree::Path` items to reduce the amount of recursion needed
    while let UseTree::Path(use_path) = tree {
        current_path.segments.push(use_path.ident.clone().into());
        tree = *use_path.tree;
    }

    match tree {
        UseTree::Name(use_name) => {
            let ident = if use_name.ident == "self" {
                match current_path.segments.last() {
                    Some(seg) => seg.ident.clone(),
                    None => {
                        return Err(syn::Error::new(use_name.ident.span(), SelfInvalid));
                    }
                }
            } else {
                current_path.segments.push(use_name.ident.clone().into());
                use_name.ident
            };
            items.push(Item::UseItem(UseItem {
                path: current_path,
                ident,
            }));
        }
        UseTree::Rename(use_rename) => {
            current_path.segments.push(use_rename.ident.into());
            items.push(Item::UseItem(UseItem {
                path: current_path,
                ident: use_rename.rename,
            }));
        }
        UseTree::Glob(use_glob) => {
            items.push(Item::UseGlob(UseGlob {
                module_path: current_path,
                star_token: use_glob.star_token,
            }));
        }
        UseTree::Group(use_group) => {
            for tree in use_group.items {
                parse_use_recurse(&current_path, tree, items)?;
            }
        }
        UseTree::Path(_) => unreachable!(),
    }
    Ok(())
}
