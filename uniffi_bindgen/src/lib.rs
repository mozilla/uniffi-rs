/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Uniffi: easily build cross-platform software components in Rust
//!
//! This is a highly-experimental crate for building cross-language software components
//! in Rust, based on things we've learned and patterns we've developed in the
//! [mozilla/application-services](https://github.com/mozilla/application-services) project.
//!
//! The idea is to let you write your code once, in Rust, and then re-use it from many
//! other programming languages via Rust's C-compatible FFI layer and some automagically
//! generated binding code. If you think of it as a kind of [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
//! wannabe, with a clunkier developer experience but support for more target languages,
//! you'll be pretty close to the mark.
//!
//! Currently supported target languages include Kotlin, Swift and Python.
//!
//! ## Usage
//
//! To build a cross-language component using `uniffi`, follow these steps.
//!
//! ### 1) Specify your Component Interface
//!
//! Start by thinking about the interface you want to expose for use
//! from other languages. Use the Interface Definition Language to specify your interface
//! in a `.udl` file, where it can be processed by the tools from this crate.
//! For example you might define an interface like this:
//!
//! ```text
//! namespace example {
//!   u32 foo(u32 bar);
//! }
//!
//! dictionary MyData {
//!   u32 num_foos;
//!   bool has_a_bar;
//! }
//! ```
//!
//! ### 2) Implement the Component Interface as a Rust crate
//!
//! With the interface, defined, provide a corresponding implementation of that interface
//! as a standard-looking Rust crate, using functions and structs and so-on. For example
//! an implementation of the above Component Interface might look like this:
//!
//! ```text
//! fn foo(bar: u32) -> u32 {
//!     // TODO: a better example!
//!     bar + 42
//! }
//!
//! struct MyData {
//!   num_foos: u32,
//!   has_a_bar: bool
//! }
//! ```
//!
//! ### 3) Generate and include component scaffolding from the UDL file
//!
//! Add to your crate `uniffi_build` under `[build-dependencies]`,
//! then add a `build.rs` script to your crate and have it call `uniffi_build::generate_scaffolding`
//! to process your `.udl` file. This will generate some Rust code to be included in the top-level source
//! code of your crate. If your UDL file is named `example.udl`, then your build script would call:
//!
//! ```text
//! uniffi_build::generate_scaffolding("src/example.udl")
//! ```
//!
//! This would output a rust file named `example.uniffi.rs`, ready to be
//! included into the code of your rust crate like this:
//!
//! ```text
//! include_scaffolding!("example");
//! ```
//!
//! ### 4) Generate foreign language bindings for the library
//!
//! You will need ensure a local `uniffi-bindgen` - see <https://mozilla.github.io/uniffi-rs/tutorial/foreign_language_bindings.html>
//! This utility provides a command-line tool that can produce code to
//! consume the Rust library in any of several supported languages.
//! It is done by calling (in kotlin for example):
//!
//! ```text
//! cargo run --bin -p uniffi-bindgen --language kotlin ./src/example.udl
//! ```
//!
//! This will produce a file `example.kt` in the same directory as the .udl file, containing kotlin bindings
//! to load and use the compiled rust code via its C-compatible FFI.
//!

#![warn(rust_2018_idioms, unused_qualifications)]
#![allow(unknown_lints)]

use anyhow::{anyhow, bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs_err::{self as fs, File};
use serde::Deserialize;
use std::fmt;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::process::Command;

mod bindgen_paths;
pub mod bindings;
pub mod interface;
pub mod library_mode;
mod loader;
pub mod macro_metadata;
pub mod pipeline;
pub mod scaffolding;

#[cfg(feature = "cargo-metadata")]
pub mod cargo_metadata;

mod crate_metadata;

use crate::interface::CallbackInterface;
use crate::interface::{
    Argument, Constructor, Enum, FfiArgument, FfiField, Field, Function, Method, Object, Record,
    Variant,
};
pub use bindgen_paths::{BindgenPaths, BindgenPathsLayer};
pub use interface::ComponentInterface;
pub use library_mode::find_components;
use scaffolding::RustScaffolding;
use uniffi_meta::Type;

pub use loader::BindgenLoader;

/// The options used when creating bindings. Named such
/// it doesn't cause confusion that it's settings specific to
/// the generator itself.
// TODO: We should try and move the public interface of the module to
// this struct. For now, only the BindingGenerator uses it.
#[derive(Debug, Default)]
pub struct GenerationSettings {
    pub out_dir: Utf8PathBuf,
    pub try_format_code: bool,
    pub cdylib: Option<String>,
}

/// A trait representing a UniFFI Binding Generator
///
/// External crates that implement binding generators, should implement this type
/// and call the [`generate_external_bindings`] using a type that implements this trait.
///
/// Deprecated: External crates are encouraged to use the `BindgenLoader` type instead, which lets
/// you control the binding generation process more directly.
pub trait BindingGenerator: Sized {
    /// Handles configuring the bindings
    type Config;

    /// Creates a new config.
    fn new_config(&self, root_toml: &toml::Value) -> Result<Self::Config>;

    /// Update the various config items in preparation to write one or more of them.
    ///
    /// # Arguments
    /// - `cdylib`: The name of the cdylib file, if known.
    /// - `library_path`: The name of library used to extract the symbols.
    /// - `components`: A mutable array of [`Component`]s to be updated.
    fn update_component_configs(
        &self,
        settings: &GenerationSettings,
        components: &mut Vec<Component<Self::Config>>,
    ) -> Result<()>;

    /// Writes the bindings to the output directory
    ///
    /// # Arguments
    /// - `components`: An array of [`Component`]s representing the items to be generated.
    /// - `out_dir`: The path to where the binding generator should write the output bindings
    fn write_bindings(
        &self,
        settings: &GenerationSettings,
        components: &[Component<Self::Config>],
    ) -> Result<()>;
}

/// A trait to alter language specific type representations.
///
/// It is meant to be implemented by each language oracle. It takes a
/// ['ComponentInterface'] and uses its own specific language adjustment
/// functions to be able to generate language specific templates.
pub trait VisitMut {
    /// Go through each `Record` of a [`ComponentInterface`] and
    /// adjust it to language specific naming conventions.
    fn visit_record(&self, record: &mut Record);

    /// Change the name of an `Object` of a [`ComponentInterface`]
    /// to language specific naming conventions.
    fn visit_object(&self, object: &mut Object);

    /// Change the name of an `Object` of a [`ComponentInterface`]
    /// to language specific naming conventions.
    fn visit_callback_interface(&self, _iface: &mut CallbackInterface) {}

    /// Change the name of a `Field` of an `Enum` `Variant`
    /// to language specific naming conventions.
    fn visit_field(&self, _field: &mut Field) {}

    /// Change the name of a `FfiField` inside a `FfiStruct`
    /// to language specific naming conventions.
    fn visit_ffi_field(&self, _ffi_field: &mut FfiField) {}

    /// Change the `Arugment` of a `FfiFunction` in the [`ComponentInterface`]
    /// to language specific naming conventions.
    fn visit_ffi_argument(&self, _ffi_argument: &mut FfiArgument) {}

    /// Change the `FfiType` objects in the [`ComponentInterface`]
    /// to language specific naming conventions.
    fn visit_ffi_type(&self, _ffi_type: &mut interface::FfiType) {}

    /// Go through each `Enum` of a [`ComponentInterface`] and
    /// adjust it to language specific naming conventions.
    fn visit_enum(&self, is_error: bool, enum_: &mut Enum);

    /// Go through each `Variant` of an `Enum` and
    /// adjust it to language specific naming conventions.
    fn visit_variant(&self, _is_error: bool, _variant: &mut Variant) {}

    /// Go through each `Type` in the `TypeUniverse` of
    /// a [`ComponentInterface`] and adjust it to language specific
    /// naming conventions.
    fn visit_type(&self, type_: &mut Type);

    /// Go through each error name in the interface and adjust it to language specific naming
    /// conventions.  The new name must match the name of the Enum/Object definition after it's
    /// visited.
    fn visit_error_name(&self, name: &mut String);

    /// Go through each `Method` of an `Object` or `CallbackInterface` and
    /// adjust it to language specific naming conventions.
    fn visit_method(&self, object_name: &str, method: &mut Method);

    /// Go through each `Argument` of a `Function` and
    /// adjust it to language specific naming conventions.
    fn visit_argument(&self, _argument: &mut Argument) {}

    /// Go through each `Constructor` of a [`ComponentInterface`] and
    /// adjust it to language specific naming conventions.
    fn visit_constructor(&self, object_name: &str, constructor: &mut Constructor);

    /// Go through each `Function` of a [`ComponentInterface`] and
    /// adjust it to language specific naming conventions.
    fn visit_function(&self, function: &mut Function);
}

/// Everything needed to generate a ComponentInterface.
#[derive(Debug)]
pub struct Component<Config> {
    pub ci: ComponentInterface,
    pub config: Config,
}

/// A trait used by the bindgen to obtain config information about a source crate
/// which was found in the metadata for the library.
///
/// This is an abstraction around needing the source directory for a crate.
/// In most cases `cargo_metadata` can be used, but this should be able to work in
/// more environments.
pub trait BindgenCrateConfigSupplier {
    /// Get a `toml::value::Table` instance for the crate.
    fn get_toml(&self, _crate_name: &str) -> Result<Option<toml::value::Table>> {
        Ok(None)
    }

    /// Get the path to the TOML file for a crate.
    ///
    /// This is usually the `uniffi.toml` path in the root of the crate source.
    fn get_toml_path(&self, _crate_name: &str) -> Option<Utf8PathBuf> {
        None
    }

    /// Obtains the contents of the named UDL file which was referenced by the type metadata.
    fn get_udl(&self, crate_name: &str, udl_name: &str) -> Result<String> {
        bail!("Crate {crate_name} has no UDL {udl_name}")
    }
}

pub struct EmptyCrateConfigSupplier;
impl BindgenCrateConfigSupplier for EmptyCrateConfigSupplier {}

impl BindgenCrateConfigSupplier for &&dyn BindgenCrateConfigSupplier {
    fn get_toml(&self, crate_name: &str) -> Result<Option<toml::value::Table>> {
        (**self).get_toml(crate_name)
    }

    fn get_toml_path(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        (**self).get_toml_path(crate_name)
    }

    fn get_udl(&self, crate_name: &str, udl_name: &str) -> Result<String> {
        (**self).get_udl(crate_name, udl_name)
    }
}

/// A convenience function for the CLI to help avoid using static libs
/// in places cdylibs are required.
pub fn is_cdylib(library_file: impl AsRef<Utf8Path>) -> bool {
    library_mode::calc_cdylib_name(library_file.as_ref()).is_some()
}

/// Generate bindings for single crate via a UDL file.
///
/// This only works if you have exactly 1 crate and no shared types.
/// It should be considered deprecated, you should use the multi-crate options instead.
///
/// # Arguments
/// - `binding_generator`: Type that implements BindingGenerator
/// - `udl_file`: The path to the UDL file
/// - `config_file_override`: The path to the configuration toml file, most likely called `uniffi.toml`. If [`None`], the function will try to guess based on the crate's root.
/// - `out_dir_override`: The path to write the bindings to. If [`None`], it will be the path to the parent directory of the `udl_file`
/// - `library_file`: The path to a dynamic library to attempt to extract the definitions from and extend the component interface with. No extensions to component interface occur if it's [`None`]
/// - `crate_name`: Override the default crate name that is guessed from UDL file path.
///
/// Deprecated: External crates are encouraged to use the `BindgenLoader` type instead, which lets
/// you control the binding generation process more directly.
pub fn generate_external_bindings<T: BindingGenerator>(
    binding_generator: &T,
    udl_file: impl AsRef<Utf8Path>,
    config_file_override: Option<impl AsRef<Utf8Path>>,
    out_dir_override: Option<impl AsRef<Utf8Path>>,
    library_file: Option<impl AsRef<Utf8Path>>,
    crate_name: Option<&str>,
    try_format_code: bool,
) -> Result<()> {
    let crate_name = crate_name
        .map(|c| Ok(c.to_string()))
        .unwrap_or_else(|| crate_name_from_cargo_toml(udl_file.as_ref()))?;
    let mut ci = parse_udl(udl_file.as_ref(), &crate_name)?;
    if let Some(ref library_file) = library_file {
        macro_metadata::add_to_ci_from_library(&mut ci, library_file.as_ref())?;
    }
    let crate_root = &guess_crate_root(udl_file.as_ref()).context("Failed to guess crate root")?;

    let config_file_override = config_file_override.as_ref().map(|p| p.as_ref());

    let config = {
        let crate_config = load_toml_file(Some(&crate_root.join("uniffi.toml")))
            .context("failed to load {crate_root}/uniffi.toml")?;
        let toml_value =
            overridden_config_value(crate_config.unwrap_or_default(), config_file_override)?;
        binding_generator.new_config(&toml_value)?
    };

    let settings = GenerationSettings {
        cdylib: match library_file {
            Some(ref library_file) => {
                library_mode::calc_cdylib_name(library_file.as_ref()).map(ToOwned::to_owned)
            }
            None => None,
        },
        out_dir: get_out_dir(
            udl_file.as_ref(),
            out_dir_override.as_ref().map(|p| p.as_ref()),
        )?,
        try_format_code,
    };

    let mut components = vec![Component { ci, config }];
    binding_generator.update_component_configs(&settings, &mut components)?;

    // need to derive ffi after the bindings have had a chance to update any types.
    components[0]
        .ci
        .derive_ffi_funcs()
        .context("Failed to derive FFI functions")?;

    binding_generator.write_bindings(&settings, &components)
}

// Generate the infrastructural Rust code for implementing the UDL interface,
// such as the `extern "C"` function definitions and record data types.
// Locates and parses Cargo.toml to determine the name of the crate.
pub fn generate_component_scaffolding(
    udl_file: &Utf8Path,
    out_dir_override: Option<&Utf8Path>,
    format_code: bool,
) -> Result<()> {
    let component = parse_udl(udl_file, &crate_name_from_cargo_toml(udl_file)?)
        .with_context(|| format!("parsing udl file {udl_file}"))?;
    generate_component_scaffolding_inner(component, udl_file, out_dir_override, format_code)
}

// Generate the infrastructural Rust code for implementing the UDL interface,
// such as the `extern "C"` function definitions and record data types, using
// the specified crate name.
pub fn generate_component_scaffolding_for_crate(
    udl_file: &Utf8Path,
    crate_name: &str,
    out_dir_override: Option<&Utf8Path>,
    format_code: bool,
) -> Result<()> {
    let component =
        parse_udl(udl_file, crate_name).with_context(|| format!("parsing udl file {udl_file}"))?;
    generate_component_scaffolding_inner(component, udl_file, out_dir_override, format_code)
}

fn generate_component_scaffolding_inner(
    component: ComponentInterface,
    udl_file: &Utf8Path,
    out_dir_override: Option<&Utf8Path>,
    format_code: bool,
) -> Result<()> {
    let file_stem = udl_file.file_stem().context("not a file")?;
    let filename = format!("{file_stem}.uniffi.rs");
    let out_path = get_out_dir(udl_file, out_dir_override)?.join(filename);
    let mut f = File::create(&out_path)?;
    write!(f, "{}", RustScaffolding::new(&component, file_stem))
        .context("Failed to write output file")?;
    if format_code {
        format_code_with_rustfmt(&out_path).context("formatting generated Rust code")?;
    }
    Ok(())
}

// Generate the bindings in the target languages that call the scaffolding
// Rust code.
///
/// Deprecated: External crates are encouraged to use the `BindgenLoader` type instead, which lets
/// you control the binding generation process more directly.
pub fn generate_bindings<T: BindingGenerator>(
    udl_file: &Utf8Path,
    config_file_override: Option<&Utf8Path>,
    binding_generator: T,
    out_dir_override: Option<&Utf8Path>,
    library_file: Option<&Utf8Path>,
    crate_name: Option<&str>,
    try_format_code: bool,
) -> Result<()> {
    generate_external_bindings(
        &binding_generator,
        udl_file,
        config_file_override,
        out_dir_override,
        library_file,
        crate_name,
        try_format_code,
    )
}

pub fn print_repr(library_path: &Utf8Path) -> Result<()> {
    let metadata = macro_metadata::extract_from_library(library_path)?;
    println!("{metadata:#?}");
    Ok(())
}

// Given the path to a UDL file, locate and parse the corresponding Cargo.toml to determine
// the library crate name.
// Note that this is largely a copy of code in uniffi_macros/src/util.rs, but sharing it
// isn't trivial and it's not particularly complicated so we've just copied it.
fn crate_name_from_cargo_toml(udl_file: &Utf8Path) -> Result<String> {
    #[derive(Deserialize)]
    struct CargoToml {
        package: Package,
        #[serde(default)]
        lib: Lib,
    }

    #[derive(Deserialize)]
    struct Package {
        name: String,
    }

    #[derive(Default, Deserialize)]
    struct Lib {
        name: Option<String>,
    }

    let file = guess_crate_root(udl_file)?.join("Cargo.toml");
    let cargo_toml_str =
        fs::read_to_string(file).context("Can't find Cargo.toml to determine the crate name")?;

    let cargo_toml = toml::from_str::<CargoToml>(&cargo_toml_str)?;

    let lib_crate_name = cargo_toml
        .lib
        .name
        .unwrap_or_else(|| cargo_toml.package.name.replace('-', "_"));

    Ok(lib_crate_name)
}

/// Guess the root directory of the crate from the path of its UDL file.
///
/// For now, we assume that the UDL file is in `./src/something.udl` relative
/// to the crate root. We might consider something more sophisticated in
/// future.
pub fn guess_crate_root(udl_file: &Utf8Path) -> Result<&Utf8Path> {
    let path_guess = udl_file
        .parent()
        .context("UDL file has no parent folder!")?
        .parent()
        .context("UDL file has no grand-parent folder!")?;
    if !path_guess.join("Cargo.toml").is_file() {
        bail!("UDL file does not appear to be inside a crate")
    }
    Ok(path_guess)
}

fn get_out_dir(udl_file: &Utf8Path, out_dir_override: Option<&Utf8Path>) -> Result<Utf8PathBuf> {
    Ok(match out_dir_override {
        Some(s) => {
            // Create the directory if it doesn't exist yet.
            fs::create_dir_all(s)?;
            s.canonicalize_utf8().context("Unable to find out-dir")?
        }
        None => udl_file
            .parent()
            .context("File has no parent directory")?
            .to_owned(),
    })
}

fn parse_udl(udl_file: &Utf8Path, crate_name: &str) -> Result<ComponentInterface> {
    let udl = fs::read_to_string(udl_file)
        .with_context(|| format!("Failed to read UDL from {udl_file}"))?;
    let group = uniffi_udl::parse_udl(&udl, crate_name)?;
    ComponentInterface::from_metadata(group)
}

fn format_code_with_rustfmt(path: &Utf8Path) -> Result<()> {
    let status = Command::new("rustfmt").arg(path).status().map_err(|e| {
        let ctx = match e.kind() {
            ErrorKind::NotFound => "formatting was requested, but rustfmt was not found",
            _ => "unknown error when calling rustfmt",
        };
        anyhow!(e).context(ctx)
    })?;
    if !status.success() {
        bail!("rustmt failed when formatting scaffolding. Note: --no-format can be used to skip formatting");
    }
    Ok(())
}

/// Load TOML from file if the file exists.
fn load_toml_file(source: Option<&Utf8Path>) -> Result<Option<toml::value::Table>> {
    if let Some(source) = source {
        if source.exists() {
            let contents =
                fs::read_to_string(source).with_context(|| format!("read file: {:?}", source))?;
            return Ok(Some(
                toml::de::from_str(&contents)
                    .with_context(|| format!("parse toml: {:?}", source))?,
            ));
        }
    }

    Ok(None)
}

/// Load the default `uniffi.toml` config, merge TOML trees with `config_file_override` if specified.
fn overridden_config_value(
    mut config: toml::value::Table,
    config_file_override: Option<&Utf8Path>,
) -> Result<toml::Value> {
    let override_config = load_toml_file(config_file_override).context("override config")?;
    if let Some(override_config) = override_config {
        merge_toml(&mut config, override_config);
    }
    Ok(toml::Value::from(config))
}

fn merge_toml(a: &mut toml::value::Table, b: toml::value::Table) {
    for (key, value) in b.into_iter() {
        match a.get_mut(&key) {
            Some(existing_value) => match (existing_value, value) {
                (toml::Value::Table(ref mut t0), toml::Value::Table(t1)) => {
                    merge_toml(t0, t1);
                }
                (v, value) => *v = value,
            },
            None => {
                a.insert(key, value);
            }
        }
    }
}

// convert `anyhow::Error` and `&str` etc to askama errors.
// should only be needed by "filters", otherwise anyhow etc work directly.
pub fn to_askama_error<T: ToString + ?Sized>(t: &T) -> askama::Error {
    askama::Error::Custom(Box::new(BindingsTemplateError(t.to_string())))
}

// Need a struct to define an error that implements std::error::Error, which neither String nor
// anyhow::Error do.
#[derive(Debug)]
struct BindingsTemplateError(String);

impl fmt::Display for BindingsTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BindingsTemplateError {}

// FIXME(HACK):
// Include the askama config file into the build.
// That way cargo tracks the file and other tools relying on file tracking see it as well.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=1774585
// In the future askama should handle that itself by using the `track_path::path` API,
// see https://github.com/rust-lang/rust/pull/84029
#[allow(dead_code)]
mod __unused {
    const _: &[u8] = include_bytes!("../askama.toml");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_guessing_of_crate_root_directory_from_udl_file() {
        // When running this test, this will be the ./uniffi_bindgen directory.
        let this_crate_root = Utf8PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

        let example_crate_root = this_crate_root
            .parent()
            .expect("should have a parent directory")
            .join("examples/arithmetic");
        assert_eq!(
            guess_crate_root(&example_crate_root.join("src/arthmetic.udl")).unwrap(),
            example_crate_root
        );

        let not_a_crate_root = &this_crate_root.join("src/templates");
        assert!(guess_crate_root(&not_a_crate_root.join("src/example.udl")).is_err());
    }

    #[test]
    fn test_merge_toml() {
        let default = r#"
            foo = "foo"
            bar = "bar"

            [table1]
            foo = "foo"
            bar = "bar"
        "#;
        let mut default = toml::de::from_str(default).unwrap();

        let override_toml = r#"
            # update key
            bar = "BAR"
            # insert new key
            baz = "BAZ"

            [table1]
            # update key
            bar = "BAR"
            # insert new key
            baz = "BAZ"

            # new table
            [table1.table2]
            bar = "BAR"
            baz = "BAZ"
        "#;
        let override_toml = toml::de::from_str(override_toml).unwrap();

        let expected = r#"
            foo = "foo"
            bar = "BAR"
            baz = "BAZ"

            [table1]
            foo = "foo"
            bar = "BAR"
            baz = "BAZ"

            [table1.table2]
            bar = "BAR"
            baz = "BAZ"
        "#;
        let expected: toml::value::Table = toml::de::from_str(expected).unwrap();

        merge_toml(&mut default, override_toml);

        assert_eq!(&expected, &default);
    }
}
