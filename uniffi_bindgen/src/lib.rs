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
//! First you will need to install `uniffi-bindgen` on your system using `cargo install uniffi_bindgen`.
//! Then add to your crate `uniffi_build` under `[build-dependencies]`.
//! Finally, add a `build.rs` script to your crate and have it call `uniffi_build::generate_scaffolding`
//! to process your `.udl` file. This will generate some Rust code to be included in the top-level source
//! code of your crate. If your UDL file is named `example.udl`, then your build script would call:
//!
//! ```text
//! uniffi_build::generate_scaffolding("./src/example.udl")
//! ```
//!
//! This would output a rust file named `example.uniffi.rs`, ready to be
//! included into the code of your rust crate like this:
//!
//! ```text
//! include!(concat!(env!("OUT_DIR"), "/example.uniffi.rs"));
//! ```
//!
//! ### 4) Generate foreign language bindings for the library
//!
//! The `uniffi-bindgen` utility provides a command-line tool that can produce code to
//! consume the Rust library in any of several supported languages.
//! It is done by calling (in kotlin for example):
//!
//! ```text
//! uniffi-bindgen --language kotlin ./src/example.udl
//! ```
//!
//! This will produce a file `example.kt` in the same directory as the .udl file, containing kotlin bindings
//! to load and use the compiled rust code via its C-compatible FFI.
//!

#![warn(rust_2018_idioms)]
#![allow(unknown_lints)]

const BINDGEN_VERSION: &str = env!("CARGO_PKG_VERSION");

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::io::prelude::*;
use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

pub mod backend;
pub mod interface;
pub mod scaffolding;

pub use interface::ComponentInterface;
use scaffolding::RustScaffolding;

/// A trait representing a Binding Generator Configuration
///
/// External crates that implement binding generators need to implement this trait and set it as
/// the `BindingGenerator.config` associated type.  `generate_external_bindings()` then uses it to
/// generate the config that's passed to `BindingGenerator.write_bindings()`
pub trait BindingGeneratorConfig: for<'de> Deserialize<'de> {
    /// Get the entry for this config from the `bindings` table.
    fn get_entry_from_bindings_table(bindings: &toml::Value) -> Option<toml::Value>;

    /// Get default config values from the `ComponentInterface`
    ///
    /// These will replace missing entries in the bindings-specific config
    fn get_config_defaults(ci: &ComponentInterface) -> Vec<(String, toml::Value)>;
}

fn load_bindings_config<BC: BindingGeneratorConfig>(
    ci: &ComponentInterface,
    udl_file: &Path,
    config_file_override: Option<&Path>,
) -> Result<BC> {
    // Load the config from the TOML value, falling back to an empty map if it doesn't exist
    let mut config_map: toml::value::Table =
        match load_bindings_config_toml::<BC>(udl_file, config_file_override)? {
            Some(value) => value
                .try_into()
                .context("Bindings config must be a TOML table")?,
            None => toml::map::Map::new(),
        };

    // Update it with the defaults from the component interface
    for (key, value) in BC::get_config_defaults(ci).into_iter() {
        config_map.entry(key).or_insert(value);
    }

    // Leverage serde to convert toml::Value into the config type
    toml::Value::from(config_map)
        .try_into()
        .context("Generating bindings config from toml::Value")
}

/// Binding generator config with no members
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct EmptyBindingGeneratorConfig;

impl BindingGeneratorConfig for EmptyBindingGeneratorConfig {
    fn get_entry_from_bindings_table(_bindings: &toml::Value) -> Option<toml::Value> {
        None
    }

    fn get_config_defaults(_ci: &ComponentInterface) -> Vec<(String, toml::Value)> {
        Vec::new()
    }
}

// EmptyBindingGeneratorConfig is a unit struct, so the `derive(Deserialize)` implementation
// expects a null value rather than the empty map that we pass it.  So we need to implement
// `Deserialize` ourselves.
impl<'de> Deserialize<'de> for EmptyBindingGeneratorConfig {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(EmptyBindingGeneratorConfig)
    }
}

// Load the binding-specific config
//
// This function calulates the location of the config TOML file, parses it, and returns the result
// as a toml::Value
//
// If there is an error parsing the file then Err will be returned. If the file is missing or the
// entry for the bindings is missing, then Ok(None) will be returned.
fn load_bindings_config_toml<BC: BindingGeneratorConfig>(
    udl_file: &Path,
    config_file_override: Option<&Path>,
) -> Result<Option<toml::Value>> {
    let config_path = match config_file_override {
        Some(cfg) => cfg.to_owned(),
        None => guess_crate_root(udl_file)?.join("uniffi.toml"),
    };

    if !config_path.exists() {
        return Ok(None);
    }

    let contents = slurp_file(&config_path)
        .with_context(|| format!("Failed to read config file from {:?}", config_path))?;
    let full_config = toml::Value::from_str(&contents)
        .with_context(|| format!("Failed to parse config file {:?}", config_path))?;

    Ok(full_config
        .get("bindings")
        .map(BC::get_entry_from_bindings_table)
        .flatten())
}

/// A trait representing a UniFFI Binding Generator
///
/// External crates that implement binding generators, should implement this type
/// and call the [`generate_external_bindings`] using a type that implements this trait.
pub trait BindingGenerator: Sized {
    /// Associated type representing a the bindings-specifig configuration parsed from the
    /// uniffi.toml
    type Config: BindingGeneratorConfig;

    /// Writes the bindings to the output directory
    ///
    /// # Arguments
    /// - `ci`: A [`ComponentInterface`] representing the interface
    /// - `config`: A instance of the BindingGeneratorConfig associated with this type
    /// - `out_dir`: The path to where the binding generator should write the output bindings
    fn write_bindings(
        &self,
        ci: ComponentInterface,
        config: Self::Config,
        out_dir: &Path,
    ) -> anyhow::Result<()>;
}

/// Generate bindings for an external binding generator
/// Ideally, this should replace the [`generate_bindings`] function below
///
/// Implements an entry point for external binding generators.
/// The function does the following:
/// - It parses the `udl` in a [`ComponentInterface`]
/// - Parses the `uniffi.toml` and loads it into the type that implements [`BindingGeneratorConfig`]
/// - Creates an instance of [`BindingGenerator`], based on type argument `B`, and run [`BindingGenerator::write_bindings`] on it
///
/// # Arguments
/// - `binding_generator`: Type that implements BindingGenerator
/// - `udl_file`: The path to the UDL file
/// - `config_file_override`: The path to the configuration toml file, most likely called `uniffi.toml`. If [`None`], the function will try to guess based on the crate's root.
/// - `out_dir_override`: The path to write the bindings to. If [`None`], it will be the path to the parent directory of the `udl_file`
pub fn generate_external_bindings(
    binding_generator: impl BindingGenerator,
    udl_file: impl AsRef<Path>,
    config_file_override: Option<impl AsRef<Path>>,
    out_dir_override: Option<impl AsRef<Path>>,
) -> Result<()> {
    let out_dir_override = out_dir_override.as_ref().map(|p| p.as_ref());
    let config_file_override = config_file_override.as_ref().map(|p| p.as_ref());
    let out_dir = get_out_dir(udl_file.as_ref(), out_dir_override)?;
    let component = parse_udl(udl_file.as_ref()).context("Error parsing UDL")?;
    let bindings_config =
        load_bindings_config(&component, udl_file.as_ref(), config_file_override)?;
    binding_generator.write_bindings(component, bindings_config, out_dir.as_path())
}

// Generate the infrastructural Rust code for implementing the UDL interface,
// such as the `extern "C"` function definitions and record data types.
pub fn generate_component_scaffolding<P: AsRef<Path>>(
    udl_file: P,
    _config_file_override: Option<P>,
    out_dir_override: Option<P>,
    format_code: bool,
) -> Result<()> {
    let out_dir_override = out_dir_override.as_ref().map(|p| p.as_ref());
    let udl_file = udl_file.as_ref();
    let component = parse_udl(udl_file)?;
    let mut filename = Path::new(&udl_file)
        .file_stem()
        .ok_or_else(|| anyhow!("not a file"))?
        .to_os_string();
    filename.push(".uniffi.rs");
    let mut out_dir = get_out_dir(udl_file, out_dir_override)?;
    out_dir.push(filename);
    let mut f =
        File::create(&out_dir).map_err(|e| anyhow!("Failed to create output file: {:?}", e))?;
    write!(f, "{}", RustScaffolding::new(&component))
        .map_err(|e| anyhow!("Failed to write output file: {:?}", e))?;
    if format_code {
        Command::new("rustfmt").arg(&out_dir).status()?;
    }
    Ok(())
}

// Generate the bindings in the target languages that call the scaffolding
// Rust code.
pub fn generate_bindings<P: AsRef<Path>>(
    udl_file: P,
    config_file_override: Option<P>,
    target_languages: Vec<&str>,
    out_dir_override: Option<P>,
    try_format_code: bool,
) -> Result<()> {
    // TODO: rework this
    Ok(())
}

/// Guess the root directory of the crate from the path of its UDL file.
///
/// For now, we assume that the UDL file is in `./src/something.udl` relative
/// to the crate root. We might consider something more sophisticated in
/// future.
fn guess_crate_root(udl_file: &Path) -> Result<&Path> {
    let path_guess = udl_file
        .parent()
        .ok_or_else(|| anyhow!("UDL file has no parent folder!"))?
        .parent()
        .ok_or_else(|| anyhow!("UDL file has no grand-parent folder!"))?;
    if !path_guess.join("Cargo.toml").is_file() {
        bail!("UDL file does not appear to be inside a crate")
    }
    Ok(path_guess)
}

fn get_out_dir(udl_file: &Path, out_dir_override: Option<&Path>) -> Result<PathBuf> {
    Ok(match out_dir_override {
        Some(s) => {
            // Create the directory if it doesn't exist yet.
            std::fs::create_dir_all(&s)?;
            s.canonicalize()
                .map_err(|e| anyhow!("Unable to find out-dir: {:?}", e))?
        }
        None => udl_file
            .parent()
            .ok_or_else(|| anyhow!("File has no parent directory"))?
            .to_owned(),
    })
}

fn parse_udl(udl_file: &Path) -> Result<ComponentInterface> {
    let udl =
        slurp_file(udl_file).map_err(|_| anyhow!("Failed to read UDL from {:?}", &udl_file))?;
    udl.parse::<interface::ComponentInterface>()
        .map_err(|e| anyhow!("Failed to parse UDL: {}", e))
}

fn slurp_file(file_name: &Path) -> Result<String> {
    let mut contents = String::new();
    let mut f = File::open(file_name)?;
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn run_main() -> Result<()> {
    const POSSIBLE_LANGUAGES: &[&str] = &["kotlin", "python", "swift", "ruby"];
    let matches = clap::App::new("uniffi-bindgen")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
        .subcommand(
            clap::SubCommand::with_name("generate")
                .about("Generate foreign language bindings")
                .arg(
                    clap::Arg::with_name("language")
                        .required(true)
                        .takes_value(true)
                        .long("--language")
                        .short("-l")
                        .multiple(true)
                        .number_of_values(1)
                        .possible_values(POSSIBLE_LANGUAGES)
                        .help("Foreign language(s) for which to build bindings"),
                )
                .arg(
                    clap::Arg::with_name("out_dir")
                        .long("--out-dir")
                        .short("-o")
                        .takes_value(true)
                        .help("Directory in which to write generated files. Default is same folder as .udl file."),
                )
                .arg(
                    clap::Arg::with_name("no_format")
                        .long("--no-format")
                        .help("Do not try to format the generated bindings"),
                )
                .arg(clap::Arg::with_name("udl_file").required(true))
                .arg(
                    clap::Arg::with_name("config")
                    .long("--config-path")
                    .takes_value(true)
                    .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("scaffolding")
                .about("Generate Rust scaffolding code")
                .arg(
                    clap::Arg::with_name("out_dir")
                        .long("--out-dir")
                        .short("-o")
                        .takes_value(true)
                        .help("Directory in which to write generated files. Default is same folder as .udl file."),
                )
                .arg(
                    clap::Arg::with_name("manifest")
                    .long("--manifest-path")
                    .takes_value(true)
                    .help("Path to crate's Cargo.toml. If not provided, Cargo.toml is assumed to be in the UDL's file parent folder.")
                )
                .arg(
                    clap::Arg::with_name("config")
                    .long("--config-path")
                    .takes_value(true)
                    .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
                )
                .arg(
                    clap::Arg::with_name("no_format")
                        .long("--no-format")
                        .help("Do not format the generated code with rustfmt (useful for maintainers)"),
                )
                .arg(clap::Arg::with_name("udl_file").required(true)),
        )
        .subcommand(
            clap::SubCommand::with_name("test")
            .about("Run test scripts against foreign language bindings")
            .arg(clap::Arg::with_name("cdylib_dir").required(true).help("Path to the directory containing the cdylib the scripts will be testing against."))
            .arg(clap::Arg::with_name("udl_file").required(true))
            .arg(clap::Arg::with_name("test_scripts").required(true).multiple(true).help("Foreign language(s) test scripts to run"))
            .arg(
                clap::Arg::with_name("config")
                .long("--config-path")
                .takes_value(true)
                .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess from the UDL's file location.")
            )
        )
        .get_matches();
    match matches.subcommand() {
        ("generate", Some(m)) => crate::generate_bindings(
            m.value_of_os("udl_file").unwrap(), // Required
            m.value_of_os("config"),
            m.values_of("language").unwrap().collect(), // Required
            m.value_of_os("out_dir"),
            !m.is_present("no_format"),
        )?,
        ("scaffolding", Some(m)) => crate::generate_component_scaffolding(
            m.value_of_os("udl_file").unwrap(), // Required
            m.value_of_os("config"),
            m.value_of_os("out_dir"),
            !m.is_present("no_format"),
        )?,
        _ => bail!("No command specified; try `--help` for some help."),
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_guessing_of_crate_root_directory_from_udl_file() {
        // When running this test, this will be the ./uniffi_bindgen directory.
        let this_crate_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

        let example_crate_root = this_crate_root
            .parent()
            .expect("should have a parent directory")
            .join("./examples/arithmetic");
        assert_eq!(
            guess_crate_root(&example_crate_root.join("./src/arthmetic.udl")).unwrap(),
            example_crate_root
        );

        let not_a_crate_root = &this_crate_root.join("./src/templates");
        assert!(guess_crate_root(&not_a_crate_root.join("./src/example.udl")).is_err());
    }
}
