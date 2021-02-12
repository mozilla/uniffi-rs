/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Macros for `uniffi`.
//!
//! Currently this is just for easily generating integration tests, but maybe
//! we'll put some other code-annotation helper macros in here at some point.

use quote::{format_ident, quote};
use std::env;
use std::path::PathBuf;
use syn::{bracketed, punctuated::Punctuated, spanned::Spanned, LitStr, Token};

use uniffi_bindgen::{
    interface::ComponentInterface,
    scaffolding::{RustScaffolding, Template},
};

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
/// To use it, invoke the macro with the interface definition file as the first
/// argument, then one or more file paths relative to the crate root directory.
/// It will produce one `#[test]` function per file, in a manner designed to
/// play nicely with `cargo test` and its test filtering options.
#[proc_macro]
pub fn build_foreign_language_testcases(paths: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let paths = syn::parse_macro_input!(paths as FilePaths);
    // We resolve each path relative to the crate root directory.
    let pkg_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Missing $CARGO_MANIFEST_DIR, cannot build tests for generated bindings");
    // For each file found, generate a matching testcase.
    let interface_file = &paths.interface_file;
    let test_functions = paths.test_scripts
        .iter()
        .map(|file_path| {
            let test_file_pathbuf: PathBuf = [&pkg_dir, file_path].iter().collect();
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
                    uniffi::testing::run_foreign_language_testcase(#pkg_dir, #interface_file, #test_file_path)
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
    interface_file: String,
    test_scripts: Vec<String>,
}

impl syn::parse::Parse for FilePaths {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let interface_file: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let array_contents;
        bracketed!(array_contents in input);
        let test_scripts = Punctuated::<LitStr, Token![,]>::parse_terminated(&array_contents)?
            .iter()
            .map(|s| s.value())
            .collect();
        Ok(FilePaths {
            interface_file: interface_file.value(),
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

/// A macro to generate and include UniFFI component scaffolding.
///
/// This macro that can be used by UniFFI component crates to automatically generate
/// and include the UniFFI component scaffolding and include it into their Rust code
/// without the need for a separate `.udl` file or a separate build step. It can be
/// be used like this:
///
/// ```rs
///
/// #[uniffi::declare_interface]
/// mod my_example {
///
///     // The Rust code to implement the component goes inside here.
///
///     pub struct Example {
///         pub fn hello(&self) -> String {
///             String::new("world")
///         }
///     }
/// }
/// ```
///
/// The contents of the inline module must be Rust code that can be parsed into a UniFFI
/// `ComponentInterface`, which means that it must satisfy a variety of restrictions so
/// that UniFFI can correctly understand it. (The precise details of those restrictions
/// are still being defined...).
///
/// After parsing the contents of the inline module, the macro will generate accompanying
/// Rust scaffolding code to expose it via an `extern "C"` FFI and will lift the resulting
/// code up to the top level of the containing module. The result makes the declared interface
/// available both to other Rust consumers (via the written Rust code), and to bindings from
/// foreign languages (via the generated FFI).
///
/// Ideally, this macro would work as an *inner* attribute on a module, so that users
/// could write this at the top-level of their Rust code rather than using an inline
/// submodule:
///
/// ```rs
///
/// #![uniffi::declare_interface]
///
/// // The Rust code to implement the component just follows the macro.
///
/// pub struct Example {
///     pub fn hello(&self) -> String {
///         String::new("world")
///     }
/// }
/// ```
///
/// Unfortunately, inner attributes at the top level of a crate are still unstable, and
/// even when enabled are quite fiddly to work with. But maybe one day..!
///
#[proc_macro_attribute]
pub fn declare_interface(
    _attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // We expect to be applied to an inline module declaration.
    let mut module = syn::parse_macro_input!(item as syn::ItemMod);
    // The contents of the module must declare a UniFFI-compatible interface.
    // TODO: In theory, we should be able to handle errors from parsing out the ComponentInterface
    // and report them to the user directly on their provided Rust code, in the same way that
    // `rustc` does. That probably requires some cleaning up of our parsing logic though.
    let ci = ComponentInterface::from_rust_module(&module)
        .expect("Failed to parse a ComponentInterface from the provided Rust code");
    // Generate the Rust scaffolding.
    // For macro purposes it would be better to generate this directly via `quote!`
    // rather than rendering to a string and re-parsing, but this'll do for now.
    // TODO: again, errors should be reported as red squiggles on the user's Rust code.
    let scaffolding = RustScaffolding::new(&ci)
        .render()
        .expect("Failed to generated Rust scaffolding");
    let scaffolding: syn::File =
        syn::parse_str(scaffolding.as_str()).expect("Failed to parse generated Rust scaffolding");
    // Append the generated scaffolding to the inline module.
    // Unwrapping is safe because, if the module didn't have content, we wouldn't have parsed it.
    module.content.as_mut().unwrap().1.extend(scaffolding.items);
    // Import the public items from the module so they're available at the top level,
    // where you might normally expect them when writing Rust by hand.
    let namespace = &module.ident;
    let imports = ci
        .iter_member_names()
        .iter()
        .map(|name| {
            let name = syn::Ident::new(name, module.span());
            quote! {
                pub use #namespace::#name;
            }
        })
        .collect::<Vec<_>>();
    proc_macro::TokenStream::from(quote! {
        #module
        #(#imports)*
    })
}
