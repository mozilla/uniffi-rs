/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::Span;
use syn::{punctuated::Punctuated, Ident, ItemUse, Token, UseTree};

use crate::{ErrorKind::*, Item, Visibility};

pub struct UseItem {
    pub vis: Visibility,
    // Path to the item being imported
    pub path: syn::Path,
    // Name of the import
    pub ident: Ident,
    pub span: Span,
}

pub struct UseGlob {
    pub vis: Visibility,
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
    parse_use_recurse(use_item.vis.into(), &path, use_item.tree, &mut items)?;
    Ok(items)
}

fn parse_use_recurse(
    vis: Visibility,
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
            let span = use_name.ident.span();
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
                vis,
                path: current_path,
                ident,
                span,
            }));
        }
        UseTree::Rename(use_rename) => {
            let span = use_rename.ident.span();
            current_path.segments.push(use_rename.ident.into());
            items.push(Item::UseItem(UseItem {
                vis,
                path: current_path,
                ident: use_rename.rename,
                span,
            }));
        }
        UseTree::Glob(use_glob) => {
            items.push(Item::UseGlob(UseGlob {
                vis,
                module_path: current_path,
                star_token: use_glob.star_token,
            }));
        }
        UseTree::Group(use_group) => {
            for tree in use_group.items {
                parse_use_recurse(vis, &current_path, tree, items)?;
            }
        }
        UseTree::Path(_) => unreachable!(),
    }
    Ok(())
}
