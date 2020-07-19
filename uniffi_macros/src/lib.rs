/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Macros for `uniffi`.
//!
//! Currently this is just for easily generating integration tests, but maybe
//! we'll put some other code-annotation helper macros in here at some point.

use quote::{format_ident, quote};
use std::convert::TryFrom;
use std::env;
use std::path::PathBuf;
use syn::{punctuated::Punctuated, LitStr, Token};

/// A macro to build testcases for a component's generated bindings.
///
/// This macro provides some plumbing to write automated tests for the generated
/// foreign language bindings of a component. As a component author, you can write
/// script files in the target foreign language(s) that exercise you component API,
/// and then call this macro to produce a `cargo test` testcase from each one.
/// The generated code will execute your script file with appropriate configuration and
/// environment to let it load the component bindings, and will pass iff the script
/// exits successfully.
///
/// To use it, invoke the macro with one or more file paths relative to the crate root
/// directory. It will produce one `#[test]` function per file, in a manner designed to
/// play nicely with `cargo test` and its test filtering options.
#[proc_macro]
pub fn build_foreign_language_testcases(paths: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let paths = syn::parse_macro_input!(paths as FilePaths).0;
    // We resolve each path relative to the crate root directory.
    let pkg_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Missing $CARGO_MANIFEST_DIR, cannot build tests for generated bindings");
    // For each file found, generate a matching testcase.
    let test_functions = paths
        .iter()
        .map(|file_path| {
            let test_file_pathbuf: PathBuf = [&pkg_dir, &file_path].iter().collect();
            let test_file_path = test_file_pathbuf.to_string_lossy();
            let test_file_name = test_file_pathbuf
                .file_name()
                .expect("Test file has no name, cannot build tests for generated bindings")
                .to_string_lossy();
            let test_name = format_ident!(
                "uniffi_foreign_language_testcase_{}",
                test_file_name.replace(|c: char| !c.is_alphanumeric(), "_")
            );
            quote! {
                #[test]
                fn #test_name () -> anyhow::Result<()> {
                    uniffi::support::tests::run_foreign_language_testcase(#pkg_dir, #test_file_path)
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();
    let test_module = quote! {
        #(#test_functions)*
    };
    proc_macro::TokenStream::from(test_module)
}

/// Newtype to simplifying parsing a list of file paths from macro input.
#[derive(Debug)]
struct FilePaths(Vec<String>);

impl syn::parse::Parse for FilePaths {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(FilePaths(
            Punctuated::<LitStr, Token![,]>::parse_terminated(input)?
                .iter()
                .map(|s| s.value())
                .collect(),
        ))
    }
}

/// A macro to statically check consistency between names in the rust layer
/// and the generated API.
///
/// Given a Rust type exported via uniffi, you can statically assert the exported
/// name of the type like this:
///
/// ```
/// uniffi_assert_type_name!(ExampleType, "ExampleType");
/// ```
///
/// If the generated API does not export that type under the given name, this
/// will generate a compile-time error. (It won't be a particularly *helpful*
/// error, unfortunately, at least until the `const_panic` feature is stabilised).
///
/// The uniffi proc macros can only operate based on local information, which
/// means that the generated API defintion for a function must declare its types
/// based on the *textual* type names used in the exported rust function. This
/// raises the risk of code like the following compiling successfully, but failing
/// to generate a consistent API definition:
///
/// ```
/// #[uniffi_export_record]
/// pub struct Example {
///     v: u32,
/// }
///
/// type Renamed = Example;
///
/// #[uniffi_export_fn]
/// pub fn my_example(arg: Renamed) -> u32 {
///   arg.v
/// }
/// ```
///
/// The generated API definition for the `my_export` function would declare an
/// argument type of `Renamed`, which does not correspond to any types in the API.
/// By emitting a static assertion about the exported name of the type, we can
/// prevent this code from compiling at all.
#[proc_macro]
pub fn uniffi_assert_type_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let TypeAndName {
        typ,
        sep_token,
        name,
    } = syn::parse_macro_input!(input as TypeAndName);
    let name = name.value();
    // We need to generate a `const` expression that is true iff the given
    // name matches the name on the `ViaFfi` trait for the given type.
    // Luckily, rust allows us to do the following in a const context:
    //  * access the length of a static string
    //  * access the underlying bytes of a static string
    //  * compare two `u8`s for equality.
    // That's enough for a very verbose byte-by-byte equality check
    // against a known string.
    let mut checks: Vec<proc_macro2::TokenStream> = vec![quote! {
        NAME.len() == #name.len()
    }];
    if name.len() > 0 {
        checks.extend(name.as_bytes().iter().enumerate().map(|(i, v)| {
            let i = syn::Index::from(i);
            quote! {
                NAME.as_bytes()[#i] == #v
            }
        }));
    }
    let assertion = quote! {
        static_assertions::const_assert!({
            const NAME: &'static str = <#typ as uniffi::support::ViaFfi>::NAME;
            #(#checks)&&*
        });
    };
    assertion.into()
}

#[derive(Debug)]
struct TypeAndName {
    typ: syn::Type,
    sep_token: Token![,],
    name: syn::LitStr,
}

impl syn::parse::Parse for TypeAndName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(TypeAndName {
            typ: input.parse()?,
            sep_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn uniffi_export_enum(
    attrs: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attrs.is_empty() {
        panic!("Macro attrs not supported, and this needs better error reporting")
    }
    let item = syn::parse_macro_input!(body as syn::ItemEnum);
    match uniffi::macros::EnumDefinition::try_from(&item) {
        Err(e) => e.to_compile_error(),
        Ok(defn) => quote! {
            #item
            #defn
        },
    }
    .into()
}

#[proc_macro_attribute]
pub fn uniffi_export_fn(
    attrs: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attrs.is_empty() {
        panic!("Macro attrs not supported, and this needs better error reporting")
    }
    let item = syn::parse_macro_input!(body as syn::ItemFn);
    match uniffi::macros::FunctionDefinition::try_from(&item) {
        Err(e) => e.to_compile_error(),
        Ok(defn) => quote! {
            #item
            #defn
        },
    }
    .into()
}

#[proc_macro_attribute]
pub fn uniffi_export_record(
    attrs: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attrs.is_empty() {
        panic!("Macro attrs not supported, and this needs better error reporting")
    }
    let item = syn::parse_macro_input!(body as syn::ItemStruct);
    let parsed = format!("RECORD {:?}", item);
    match uniffi::macros::RecordDefinition::try_from(&item) {
        Err(e) => e.to_compile_error(),
        Ok(defn) => quote! {
            #[doc=#parsed]
            #item
            #defn
        },
    }
    .into()
}

#[proc_macro_attribute]
pub fn uniffi_export_class(
    attrs: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attrs.is_empty() {
        panic!("Macro attrs not supported, and this needs better error reporting")
    }
    let item = syn::parse_macro_input!(body as syn::ItemStruct);
    let parsed = format!("CLASS {:?}", item);
    match uniffi::macros::RecordDefinition::try_from(&item) {
        Err(e) => e.to_compile_error(),
        Ok(defn) => quote! {
            #[doc=#parsed]
            #item
            #defn
        },
    }
    .into()
}

#[proc_macro_attribute]
pub fn uniffi_export_methods(
    attrs: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attrs.is_empty() {
        panic!("Macro attrs not supported, and this needs better error reporting")
    }
    let item = syn::parse_macro_input!(body as syn::ItemImpl);
    let parsed = format!("METHODS {:?}", item);
    match uniffi::macros::MethodDefinitions::try_from(&item) {
        Err(e) => e.to_compile_error(),
        Ok(defns) => quote! {
            #[doc=#parsed]
            #item
            #defns
        },
    }
    .into()
}
