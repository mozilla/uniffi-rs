/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{env, io};

use camino::{Utf8Path, Utf8PathBuf};
use fs_err::{self as fs};
use once_cell::sync::Lazy;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
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
    let type_path = type_as_type_path(ty)?;

    if type_path.qself.is_some() {
        return Err(syn::Error::new_spanned(
            type_path,
            "qualified self types are not currently supported by uniffi::export",
        ));
    }

    if type_path.path.segments.len() > 1 {
        return Err(syn::Error::new_spanned(
            &type_path,
            "qualified paths in types are not currently supported by uniffi::export",
        ));
    }

    match &type_path.path.segments.first() {
        Some(seg) => match &seg.arguments {
            syn::PathArguments::None => convert_bare_type_name(&seg.ident),
            syn::PathArguments::AngleBracketed(a) => convert_generic_type(&seg.ident, a),
            syn::PathArguments::Parenthesized(_) => Err(type_not_supported(type_path)),
        },
        None => Err(syn::Error::new_spanned(
            &type_path,
            "unreachable: TypePath must have non-empty segments",
        )),
    }
}

fn convert_generic_type(
    ident: &Ident,
    a: &syn::AngleBracketedGenericArguments,
) -> syn::Result<Type> {
    let mut it = a.args.iter();
    match it.next() {
        // `u8<>` is a valid way to write `u8` in the type namespace, so why not?
        None => convert_bare_type_name(ident),
        Some(arg1) => match it.next() {
            None => convert_generic_type1(ident, arg1),
            Some(arg2) => match it.next() {
                None => convert_generic_type2(ident, arg1, arg2),
                Some(_) => Err(syn::Error::new_spanned(
                    ident,
                    "types with more than two generics are not currently
                     supported by uniffi::export",
                )),
            },
        },
    }
}

fn convert_bare_type_name(ident: &Ident) -> syn::Result<Type> {
    match ident.to_string().as_str() {
        "u8" => Ok(Type::U8),
        "u16" => Ok(Type::U16),
        "u32" => Ok(Type::U32),
        "u64" => Ok(Type::U64),
        "i8" => Ok(Type::I8),
        "i16" => Ok(Type::I16),
        "i32" => Ok(Type::I32),
        "i64" => Ok(Type::I64),
        "f32" => Ok(Type::F32),
        "f64" => Ok(Type::F64),
        "bool" => Ok(Type::Bool),
        "String" => Ok(Type::String),
        _ => Err(type_not_supported(ident)),
    }
}

fn convert_generic_type1(ident: &Ident, arg: &syn::GenericArgument) -> syn::Result<Type> {
    let arg = arg_as_type(arg)?;
    match ident.to_string().as_str() {
        "Option" => Ok(Type::Option {
            inner_type: convert_type(arg)?.into(),
        }),
        "Vec" => Ok(Type::Vec {
            inner_type: convert_type(arg)?.into(),
        }),
        _ => Err(type_not_supported(ident)),
    }
}

fn convert_generic_type2(
    ident: &Ident,
    arg1: &syn::GenericArgument,
    arg2: &syn::GenericArgument,
) -> syn::Result<Type> {
    let arg1 = arg_as_type(arg1)?;
    let arg2 = arg_as_type(arg2)?;

    match ident.to_string().as_str() {
        "HashMap" => Ok(Type::HashMap {
            key_type: convert_type(arg1)?.into(),
            value_type: convert_type(arg2)?.into(),
        }),
        _ => Err(type_not_supported(ident)),
    }
}

fn type_as_type_path(ty: &syn::Type) -> syn::Result<&syn::TypePath> {
    match ty {
        syn::Type::Group(g) => type_as_type_path(&g.elem),
        syn::Type::Paren(p) => type_as_type_path(&p.elem),
        syn::Type::Path(p) => Ok(p),
        _ => Err(type_not_supported(ty)),
    }
}

fn arg_as_type(arg: &syn::GenericArgument) -> syn::Result<&syn::Type> {
    match arg {
        syn::GenericArgument::Type(t) => Ok(t),
        _ => Err(syn::Error::new_spanned(
            arg,
            "non-type generic parameters are not currently supported by uniffi::export",
        )),
    }
}

fn type_not_supported(ty: &impl ToTokens) -> syn::Error {
    syn::Error::new_spanned(
        &ty,
        "this type is not currently supported by uniffi::export in this position",
    )
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
