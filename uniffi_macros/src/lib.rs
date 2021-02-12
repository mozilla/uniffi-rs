/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Macros for `uniffi`.
//!
//! Currently this is just for easily generating integration tests, but maybe
//! we'll put some other code-annotation helper macros in here at some point.

use quote::{format_ident, quote};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use syn::{bracketed, punctuated::Punctuated, LitStr, Token};

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
/// To use it, invoke the macro with the udl file as the first argument, then
/// one or more file paths relative to the crate root directory.
/// It will produce one `#[test]` function per file, in a manner designed to
/// play nicely with `cargo test` and its test filtering options.
#[proc_macro]
pub fn build_foreign_language_testcases(paths: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let paths = syn::parse_macro_input!(paths as FilePaths);
    // We resolve each path relative to the crate root directory.
    let pkg_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Missing $CARGO_MANIFEST_DIR, cannot build tests for generated bindings");
    // For each file found, generate a matching testcase.
    let udl_file = &paths.udl_file;
    let test_functions = paths.test_scripts
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
                fn #test_name () -> uniffi::deps::anyhow::Result<()> {
                    uniffi::testing::run_foreign_language_testcase(#pkg_dir, #udl_file, #test_file_path)
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
struct FilePaths {
    udl_file: String,
    test_scripts: Vec<String>,
}

impl syn::parse::Parse for FilePaths {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let udl_file: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let array_contents;
        bracketed!(array_contents in input);
        let test_scripts = Punctuated::<LitStr, Token![,]>::parse_terminated(&array_contents)?
            .iter()
            .map(|s| s.value())
            .collect();
        Ok(FilePaths {
            udl_file: udl_file.value(),
            test_scripts,
        })
    }
}

/// A helper macro to include generated component scaffolding.
///
/// This is a simple convenience macro to include the UniFFI component
/// scaffolding as built by `uniffi_build::generate_scaffolding`.
/// Use it like so:
///
/// ```rs
/// uniffi_macros::include_scaffolding!("my_component_name");
/// ```
///
/// This will expand to the appropriate `include!` invocation to include
/// the generated `my_component_name.uniffi.rs` (which it assumes has
/// been successfully built by your crate's `build.rs` script).
//
#[proc_macro]
pub fn include_scaffolding(component_name: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let name = syn::parse_macro_input!(component_name as syn::LitStr);
    if std::env::var("OUT_DIR").is_err() {
        quote! {
            compile_error!("This macro assumes the crate has a build.rs script, but $OUT_DIR is not present");
        }
    } else {
        quote! {
            include!(concat!(env!("OUT_DIR"), "/", #name, ".uniffi.rs"));
        }
    }.into()
}

/// A (very naughty) macro to generate and include UniFFI component scaffolding.
///
/// This is a (very naughty) macro that can be used by UniFFI component crates to
/// automatically generate the UniFFI component scaffolding and include it into their
/// Rust code. It's intended as a simpler alternative to a build script that runs
/// `uniffi-bindgen`, and can be used like this:
///
/// ```rs
///
/// #[uniffi_macros::declare_interface]
/// mod my_example {
///     // The Rust code to implement the component goes here.
///     pub struct Example {
///         pub fn hello(&self) -> String {
///             String::new("world")
///         }
///     }
/// }
/// ```
///
/// This macro call would read a UniFFI component interface definition from `my_example.udl`,
/// generate the corresponding Rust scaffolding code, combine it with the Rust code from the
/// inline module, and make the result available as top-level items in the containing file.
///
/// What's so naughty about this, you ask? Well, the ideal macro is supposed to be a pure
/// function of its input tokens to its output tokens, but this one does some very
/// frowned-upon things behind the scenes:
///
///  * it accesses a separate file in order to find the component interface definition.
///  * it runs `uniffi-bindgen` and writes its output to disk.
///  * it parses the resulting file back into memory to include in its return value.
///
/// All in all, some pretty weird behaviours for a macro. However, you can imagine it
/// evolving to do fewer of these things in the future.
///
/// A future version could do the scaffolding generation in-process rather than
/// shelling out to `uniffi-bindgen`. A future version could directly generate a Rust
/// tokenstream rather than re-parsing the scaffolding from a string. And a far, far
/// future version could conceivably even parse the attributed Rust code in order to
/// determine the interface of the component, rather than depending on a separate `.udl`
/// file at all...
///
/// For now, consider this an experiment to see whether we like the macro syntax
/// from a developer-experience perspective.
///
#[proc_macro_attribute]
pub fn declare_interface(
    _attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // We expect to be applied to an inline module declaration, whose name
    // is the name of the `.udl` file to load.
    let module = syn::parse_macro_input!(item as syn::ItemMod);
    let component_name = module.ident.to_string();
    // Locate file relative to `src` directory of current crate.
    let file_name = format!("{}.udl", component_name);
    let pkg_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Missing $CARGO_MANIFEST_DIR, cannot generate scaffolding for current crate");
    let file_path: PathBuf = [&pkg_dir, "src", &file_name].iter().collect();
    // Shell out to the uniffi-bindgen build process.
    // Note that `uniffi_bindgen` expects to be writing into $OUT_DIR, but we can't assume that exists
    // (it only seems to be present when the crate has a build script) and so need to create a tempdir.
    let out_dir = tempfile::tempdir()
        .expect("Could not generate temporary directly in which to run uniffi-bindgen");
    run_uniffi_bindgen_scaffolding(out_dir.path().as_os_str(), file_path.as_os_str());
    // Slurp the resulting code back in, so we can include it in our output.
    let mut scaffolding = String::new();
    let mut f = File::open(out_dir.path().join(format!("{}.uniffi.rs", component_name)))
        .expect("Failed to open uniffi-bindgen output file");
    f.read_to_string(&mut scaffolding)
        .expect("Failed to read uniffi-bindgen output file");
    let scaffolding: syn::File =
        syn::parse_str(scaffolding.as_str()).expect("Failed to parse uniffi-bindgen output file");
    // Lift the contents of the inline module up to the top level.
    let mut module_items = match module.content {
        None => vec![],
        Some((_, items)) => items,
    };
    // Include the generated scaffolding.
    module_items.extend(scaffolding.items);
    // And we're done!
    proc_macro::TokenStream::from(quote! {
        #(#module_items)*
    })
}

#[cfg(not(feature = "builtin-bindgen"))]
fn run_uniffi_bindgen_scaffolding(out_dir: &OsStr, udl_file: &OsStr) -> () {
    let status = std::process::Command::new("uniffi-bindgen")
        .arg("scaffolding")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(udl_file)
        .status()
        .expect("failed to run `uniffi-bindgen` - have you installed it via `cargo install uniffi_bindgen`?");
    if !status.success() {
        panic!("Eror while generating scaffolding code with uniffi-bindgen");
    }
}

#[cfg(feature = "builtin-bindgen")]
fn run_uniffi_bindgen_scaffolding(out_dir: &OsStr, udl_file: &OsStr) -> () {
    uniffi_bindgen::generate_component_scaffolding(udl_file, None, Some(out_dir), None, false)
        .expect("Failed to generate scaffolding code");
}
