/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{env, io};

use camino::{Utf8Path, Utf8PathBuf};
use fs_err::{self as fs};
use once_cell::sync::Lazy;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::Serialize;
use tempfile::NamedTempFile;
use uniffi_meta::Type;

use super::ExportItem;

mod function;

use self::function::gen_fn_metadata;

static METADATA_DIR: Lazy<Utf8PathBuf> = Lazy::new(|| {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
    let metadata_dir = Utf8Path::new(&manifest_dir)
        .join(".uniffi")
        .join("metadata");

    fs::create_dir_all(&metadata_dir).unwrap();
    metadata_dir
});

pub(super) fn gen_metadata(item: syn::Item, mod_path: &[String]) -> syn::Result<ExportItem> {
    match item {
        syn::Item::Fn(item) => gen_fn_metadata(item, mod_path),
        syn::Item::Impl(_) => Err(syn::Error::new(
            Span::call_site(),
            "impl blocks are not yet supported",
        )),
        // FIXME: Support const / static?
        _ => Err(syn::Error::new(
            Span::call_site(),
            "unsupported item: only functions and impl \
             blocks may be annotated with this attribute",
        )),
    }
}

fn convert_type(ty: &syn::Type) -> syn::Result<Type> {
    match ty {
        syn::Type::Group(g) => convert_type(&g.elem),
        syn::Type::Path(p) => {
            if p.qself.is_some() {
                return Err(syn::Error::new_spanned(
                    p,
                    "qualified self types are not currently supported by uniffi::export",
                ));
            }

            let ty = match p.path.get_ident().map(|ident| ident.to_string()).as_deref() {
                Some("u8") => Type::U8,
                Some("u16") => Type::U16,
                Some("u32") => Type::U32,
                Some("u64") => Type::U64,
                Some("i8") => Type::I8,
                Some("i16") => Type::I16,
                Some("i32") => Type::I32,
                Some("i64") => Type::I64,
                Some("f32") => Type::F32,
                Some("f64") => Type::F64,
                Some("bool") => Type::Bool,
                Some("String") => Type::String,
                _ => {
                    return Err(syn::Error::new_spanned(
                        &p.path,
                        "this type is not currently supported by uniffi::export",
                    ));
                }
            };

            Ok(ty)
        }
        _ => Err(syn::Error::new_spanned(
            ty,
            "unsupported type syntax for uniffi::export",
        )),
    }
}

fn write_json_metadata<T: Serialize>(path: &Utf8Path, val: &T) -> io::Result<TokenStream> {
    // The usage of a `NamedTempFile` ensures that the file ends up in a consistent state if
    // multiple processes or threads run this function in parallel with the same path (for example
    // one `rustc` process and one `rust-analyzer` process).
    //
    // Without this, two serializers wanting to write to the same file could have their outputs
    // merged together by interleaved writes. By first writing to a temporary file that is then
    // atomically renamed to the desired file name, this is avoided.
    let file = NamedTempFile::new_in(
        path.parent()
            .expect("paths passed to write_json_metadata must have a parent"),
    )?;
    serde_json::to_writer_pretty(&file, val)?;
    file.persist(path)?;

    // This configures the compiler to re-run this macro invocation when the file at `path` is
    // changed (practically this should only happen if it is deleted).
    //
    // Ideally this function would return `io::Result<()>` and use a proc_macro function for
    // "tracking" the file instead of returning this TokenStream. That API is still nightly-only
    // and subject to change though, see https://github.com/rust-lang/rust/issues/73921
    let path = path.as_str();
    Ok(quote! {
        const _: &str = ::std::include_str!(#path);
    })
}
