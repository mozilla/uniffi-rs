/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{punctuated::Punctuated, spanned::Spanned, Ident, ItemUse, Path, Token, UseTree};

use crate::{files::FileId, Error, ErrorKind::*, Result};

/// Determine if an ident may come from a use tree.
///
/// This finds use statements that import the item directly (`use foo::bar`) as well as glob use
/// statements (`use foo::*`).
pub fn parse_use(source: FileId, use_item: &ItemUse, ident: &Ident) -> Result<Use> {
    let mut ctx = ParseUseContext {
        source,
        current_path: Path {
            leading_colon: use_item.leading_colon,
            segments: Punctuated::new(),
        },
        matched_path: None,
        matched_glob_paths: vec![],
    };
    parse_use_recurse(&mut ctx, &use_item.tree, ident)?;
    if let Some(p) = ctx.matched_path {
        Ok(Use::Path(p))
    } else if !ctx.matched_glob_paths.is_empty() {
        Ok(Use::GlobPaths(ctx.matched_glob_paths))
    } else {
        Ok(Use::None)
    }
}

pub enum Use {
    /// A Specific path
    Path(Path),
    /// Glob import paths
    ///
    /// The paths are the paths that we should try to import.
    /// If the import fails, ignore any errors it just means the `*` didn't match the ident we're
    /// trying to use.
    GlobPaths(Vec<(Path, Token![*])>),
    /// Nothing matched
    None,
}

fn parse_use_recurse(ctx: &mut ParseUseContext, mut tree: &UseTree, ident: &Ident) -> Result<()> {
    // Group multiple paths together to reduce the amount of recursion needed
    let mut pushed_segments = 0;
    while let UseTree::Path(use_path) = tree {
        ctx.current_path
            .segments
            .push(use_path.ident.clone().into());
        pushed_segments += 1;
        tree = &use_path.tree;
    }

    match tree {
        UseTree::Name(use_name) => {
            if use_name.ident == "self" {
                let Some(last_segment) = ctx.current_path.segments.last() else {
                    return Err(Error::new(ctx.source, use_name.ident.span(), SelfInvalid));
                };
                if last_segment.ident == *ident {
                    ctx.set_matched_path(ctx.current_path.clone())?;
                }
            } else if use_name.ident == *ident {
                let mut path = ctx.current_path.clone();
                path.segments.push(use_name.ident.clone().into());
                ctx.set_matched_path(path)?;
            }
        }
        UseTree::Rename(use_rename) => {
            if use_rename.rename == *ident {
                let mut path = ctx.current_path.clone();
                path.segments.push(use_rename.ident.clone().into());
                ctx.set_matched_path(path)?;
            }
        }
        UseTree::Glob(use_glob) => {
            let mut path = ctx.current_path.clone();
            path.segments.push(ident.clone().into());
            ctx.matched_glob_paths.push((path, use_glob.star_token));
        }
        UseTree::Group(use_group) => {
            for tree in use_group.items.iter() {
                parse_use_recurse(ctx, tree, ident)?;
            }
        }
        UseTree::Path(_) => unreachable!(),
    }

    for _ in 0..pushed_segments {
        ctx.current_path.segments.pop();
    }
    Ok(())
}

struct ParseUseContext {
    source: FileId,
    current_path: Path,
    matched_path: Option<Path>,
    matched_glob_paths: Vec<(Path, Token![*])>,
}

impl ParseUseContext {
    fn set_matched_path(&mut self, path: Path) -> Result<()> {
        if self.matched_path.is_some() {
            Err(Error::new(self.source, path.span(), NameConflict))
        } else {
            self.matched_path = Some(path);
            Ok(())
        }
    }
}
