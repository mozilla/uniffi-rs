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

fn meta_matches_uniffi_export(meta: &Meta, export_name: &str) -> bool {
    path_matches_uniffi_ident(meta.path(), export_name)
}

fn find_uniffi_derive(meta: &Meta, derive_name: &str) -> Option<DeriveInfo> {
    let mut result = None;
    if meta.path().is_ident("derive") {
        if let Meta::List(list) = meta {
            // Ignore errors when parsing the meta, maybe it's a derive syntax that we don't
            // understand
            let _ = list.parse_nested_meta(|meta| {
                if path_matches_uniffi_ident(&meta.path, derive_name) {
                    result = Some(DeriveInfo {
                        remote: false,
                        span: meta.path.span(),
                    });
                }
                Ok(())
            });
        }
    } else if path_matches_uniffi_ident(meta.path(), "remote") {
        if let Meta::List(list) = meta {
            // Ignore errors when parsing the meta, maybe it's a derive syntax that we don't
            // understand
            let _ = list.parse_nested_meta(|meta| {
                if path_matches_uniffi_ident(&meta.path, derive_name) {
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

fn path_matches_uniffi_ident(path: &Path, ident: &str) -> bool {
    let s = &path.segments;
    if s.len() == 1 && s[0].ident == ident {
        true
    } else {
        s.len() == 2 && s[0].ident == "uniffi" && s[1].ident == ident
    }
}
