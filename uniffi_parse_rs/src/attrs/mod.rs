/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod callables;
mod defaults;
mod docstring;
mod enums;
mod objects;
mod records;
mod traits;
mod utraits;

use proc_macro2::Span;
use syn::{spanned::Spanned, Meta, Path};

pub use callables::{ConstructorAttributes, FunctionAttributes, MethodAttributes};
pub use defaults::{Default, DefaultMap};
pub use docstring::extract_docstring;
pub use enums::{EnumAttributes, VariantAttributes};
pub use objects::{ImplAttributes, ObjectAttributes};
pub use records::{FieldAttributes, RecordAttributes};
pub use traits::{TraitAttributes, TraitExportType};
pub use utraits::UniffiTraitAttrs;

use crate::{paths::LookupCache, BuiltinItem, Ir, Item, Namespace, RPath};

fn meta_matches_uniffi_attr(meta: &Meta, export_name: &str) -> bool {
    let path = meta.path();
    path.segments.len() == 2
        && path.segments[0].ident == "uniffi"
        && path.segments[1].ident == export_name
}

fn meta_is_uniffi_export<'ir>(
    module_path: &RPath<'ir>,
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    meta: &Meta,
) -> bool {
    resolved_path_is_uniffi_macro(module_path, ir, cache, meta.path(), "export")
}

fn find_uniffi_derive<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    module_path: &RPath<'ir>,
    meta: &Meta,
    derive_name: &str,
) -> Option<DeriveInfo> {
    let mut result = None;
    if meta.path().is_ident("derive") {
        if let Meta::List(list) = meta {
            // Ignore errors when parsing the meta, maybe it's a derive syntax that we don't
            // understand
            let _ = list.parse_nested_meta(|meta| {
                if resolved_path_is_uniffi_derive(module_path, ir, cache, &meta.path, derive_name) {
                    result = Some(DeriveInfo {
                        remote: false,
                        span: meta.path.span(),
                    });
                }
                Ok(())
            });
        }
    } else if resolved_path_is_uniffi_macro(module_path, ir, cache, meta.path(), "remote") {
        if let Meta::List(list) = meta {
            // Ignore errors when parsing the meta, maybe it's a derive syntax that we don't
            // understand
            let _ = list.parse_nested_meta(|meta| {
                if meta.path.is_ident(derive_name) {
                    result = Some(DeriveInfo {
                        remote: true,
                        span: meta.path.span(),
                    });
                }
                Ok(())
            });
        }
    }
    result
}

pub struct DeriveInfo {
    remote: bool,
    span: Span,
}

fn resolved_path_is_uniffi_macro<'ir>(
    module_path: &RPath<'ir>,
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    path: &Path,
    name: &str,
) -> bool {
    let resolved = module_path
        .resolve(ir, cache, path, Namespace::Macro)
        .and_then(|rpath| rpath.item());
    match resolved {
        Ok(Item::Builtin(BuiltinItem::UniffiMacro(n))) if *n == name => true,
        _ => false,
    }
}

fn resolved_path_is_uniffi_derive<'ir>(
    module_path: &RPath<'ir>,
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    path: &Path,
    name: &str,
) -> bool {
    let resolved = module_path
        .resolve(ir, cache, path, Namespace::Macro)
        .and_then(|rpath| rpath.item());
    match resolved {
        Ok(Item::Builtin(BuiltinItem::UniffiDerive(n))) if *n == name => true,
        _ => false,
    }
}
