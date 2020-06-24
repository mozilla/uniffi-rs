/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Uniffi: easily build cross-language software components in Rust
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
//! Currently supported target languages include Kotlin and Python.
//!
//! ## Usage
//
//! To build a cross-language component using `uniffi`, follow these steps.
//!
//! ### 1) Specify your Component Interface
//!
//! Start by thinking about the interface you want to expose for use
//! from other languages. Use the Interface Definition Language to specify your interface
//! in a `.idl` file, where it can be processed by the tools from this crate.
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
//! fn foo(bar: u32): u32 {
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
//! ### 3) Generate and include component scaffolding from the IDL file
//!
//! With the implementation of your component in place, add a `build.rs` script to your crate
//! and have it call [generate_component_scaffolding](crate::generate_component_scaffolding)
//! to process your `.idl` file. This will generate some rust code to be included in the top-level source
//! code of your crate. If your IDL file is named `example.idl`, then your build script would call:
//!
//! ```text
//! uniffi::generate_component_scaffolding("./src/arithmetic.idl")
//! ```
//!
//! This would output a rust file named `example.uniffi.rs`, ready to be
//! included into the code of your rust crate like this:
//!
//! ```text
//! include!(concat!(env!("OUT_DIR"), "/example.uniffi.rs"));
//! ```
//!
//! ### 4) Build your crate as a `cdylib`
//!
//! The generated code automatically declares the necessary `pub extern "C"` functions and type
//! conversions to expose your rust code over a C-compatible foreign function interface.
//! Make it available for use by building it as a `cdylib` whose name is prefixed with the
//! string "uniffi_". For this example component, the necessary config in `Cargo.toml` would
//! be:
//!
//! ```text
//! [lib]
//! crate-type = ["cdylib"]
//! name = "uniffi_example"
//! ```
//!
//! Running `cargo build` on your crate should produce an appropriate shared library for your
//! system, in this case something like `libuniffi_example.so` or `libuniff_example.dylib`.
//! In addition to the compiled Rust code, this shared library will contain a copy of the abstract
//! interface definitions from your IDL file, allowing it to be safely consumed from other
//! languages.
//!
//! ### 5) Generate foreign language bindings for the library
//!
//! The `uniffi` crate provides a command-line tool that can take the dynamic library built
//! from a rust component, extract the embedded IDL details, and produce produce code for
//! consuming that library in any of several supported languages.
//!
//! To help ensure this is done with appropriate config for your component, add a `src/main.rs`
//! script to call `uniffi::run_bindgen_for_component`, like this:
//!
//! ```text
//! fn main() {
//!    uniffi::run_bindgen_for_component("example").unwrap();
//!}
//! ```
//!
//! Then you can generate bindings using `cargo run generate` in your crate. For example, to
//! generate python bindings for the example component, run:
//!
//! ```text
//! cargo run generate -l python
//! ```
//!
//! This will produce a file `example.py` in the cargo target directory, containing python bindings
//! to load and use the compiled rust code via its C-compatible FFI.
//!

use std::io::prelude::*;
use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub mod bindings;
pub mod interface;
pub mod scaffolding;
pub mod support;

use scaffolding::RustScaffolding;

pub(crate) fn slurp_file(file_name: &str) -> Result<String> {
    let mut contents = String::new();
    let mut f = File::open(file_name)?;
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

// Call this when building the rust crate that implements the specified interface.
// It will generate a bunch of the infrastructural rust code for implementing
// the interface, such as the `extern "C"` function definitions and record data types.

pub fn generate_component_scaffolding(idl_file: &str) -> Result<()> {
    println!("cargo:rerun-if-changed={}", idl_file);
    let idl = slurp_file(idl_file)
        .map_err(|_| anyhow::anyhow!("Failed to read IDL from {}", &idl_file))?;
    let component = idl
        .parse::<interface::ComponentInterface>()
        .map_err(|e| anyhow::anyhow!("Failed to parse IDL: {}", e))?;
    let mut filename = Path::new(idl_file)
        .file_stem()
        .ok_or_else(|| anyhow!("not a file"))?
        .to_os_string();
    filename.push(".uniffi.rs");
    let mut out_file =
        PathBuf::from(env::var("OUT_DIR").map_err(|_| anyhow::anyhow!("No $OUT_DIR specified"))?);
    out_file.push(filename);
    let mut f =
        File::create(out_file).map_err(|e| anyhow!("Failed to create output file: {:?}", e))?;
    write!(f, "{}", RustScaffolding::new(&component))
        .map_err(|e| anyhow!("Failed to write output file: {:?}", e))
}

// Call this when generating forgein language bindings from the command-line.

pub fn run_bindgen_for_component(component_name: &str) -> Result<()> {
    run_bindgen_helper(Some(component_name))
}

pub fn run_bindgen_command() -> Result<()> {
    run_bindgen_helper(None)
}

const POSSIBLE_LANGUAGES: [&str; 2] = ["kotlin", "python"];

pub fn run_bindgen_helper(component_name: Option<&str>) -> Result<()> {
    let default_lib_file = match component_name {
        None => None,
        Some(component_name) => Some(resolve_default_library_file(component_name)?),
    };
    let app = clap::App::new("uniffi")
        .about("Foreign language bindings generator for Rust")
        .subcommand(
            clap::SubCommand::with_name("generate")
                .about("Generate foreign language bindings")
                .arg(
                    clap::Arg::with_name("language")
                        .takes_value(true)
                        .long("--language")
                        .short("-l")
                        .multiple(true)
                        .number_of_values(1)
                        .possible_values(&POSSIBLE_LANGUAGES)
                        .help("Foreign language(s) for which to build bindings"),
                )
                .arg(
                    clap::Arg::with_name("lib_file")
                        .takes_value(true)
                        .required(if let Some(_) = default_lib_file {
                            false
                        } else {
                            true
                        })
                        .help("compiled uniffi library to generate bindings for"),
                )
                .arg(
                    clap::Arg::with_name("out_dir")
                        .takes_value(true)
                        .help("directory in which to write generated files"),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("exec")
                .about("Execute foreign language code with component bindings")
                .arg(
                    clap::Arg::with_name("language")
                        .takes_value(true)
                        .long("--language")
                        .short("-l")
                        .possible_values(&POSSIBLE_LANGUAGES)
                        .help("Foreign language interpreter to invoke"),
                )
                .arg(
                    clap::Arg::with_name("script")
                        .takes_value(true)
                        .help("files to execute"),
                ),
        );

    let matches = app.get_matches();
    match matches.subcommand() {
        ("generate", Some(m)) => run_bindings_generate_subcommand(default_lib_file, m)?,
        ("exec", Some(m)) => run_bindings_exec_subcommand(component_name.is_some(), m)?,
        _ => println!("No command specified; try `--help` for some help."),
    }
    Ok(())
}

fn run_bindings_generate_subcommand(
    default_lib_file: Option<std::ffi::OsString>,
    command_args: &clap::ArgMatches,
) -> Result<()> {
    let lib_file = match command_args.value_of_os("lib_file") {
        Some(lib_file) => lib_file.to_os_string(),
        None => match default_lib_file {
            None => bail!("No lib_file specified and no default provided; this should be impossible but here we are..."),
            Some(lib_file) => lib_file,
        }
    };
    let out_dir = match command_args.value_of_os("out_dir") {
        Some(dir) => dir.to_os_string(),
        None => PathBuf::from(&lib_file)
            .parent()
            .ok_or_else(|| anyhow!("Library file has no parent directory"))?
            .as_os_str()
            .to_os_string(),
    };
    println!("Extracting Interface Definition from {:?}", lib_file);
    let ci = get_component_interface_from_library(&lib_file)?;
    let languages: Vec<&str> = match command_args.values_of("language") {
        None => POSSIBLE_LANGUAGES.iter().cloned().collect(),
        Some(ls) => ls.collect(),
    };
    for lang in languages {
        match &lang {
            &"kotlin" => {
                println!(
                    "Generating Kotlin bindings into {}",
                    out_dir.to_str().unwrap_or("[UNPRINTABLE]")
                );
                bindings::kotlin::compile_kotlin_bindings(&ci, &out_dir)?;
            }
            &"python" => {
                println!(
                    "Generating Python bindings {}",
                    out_dir.to_str().unwrap_or("[UNPRINTABLE]")
                );
                bindings::python::write_python_bindings(&ci, &out_dir)?;
            }
            _ => bail!(
                "Somehow tried to generate bindings for unsupported language {}",
                lang
            ),
        }
    }
    println!("Done!");
    Ok(())
}

fn run_bindings_exec_subcommand(
    use_target_dir: bool,
    command_args: &clap::ArgMatches,
) -> Result<()> {
    let curdir = current_target_dir()?;
    let target_dir = if use_target_dir {
        Some(curdir.as_os_str())
    } else {
        None
    };
    let script_file = command_args.value_of("script");
    let lang = match command_args.value_of("language") {
        Some(lang) => lang,
        None => {
            // Try to guess language based on script file extension.
            if let None = script_file {
                bail!("No script file and no language specified, so I don't know what language shell to start")
            }
            let script_file_buf = PathBuf::from(script_file.unwrap());
            let ext = script_file_buf.extension().unwrap_or_default();
            if ext == "kts" {
                "kotlin"
            } else if ext == "py" {
                "python"
            } else {
                bail!("Cannot guess language of script file, please specify it explicitly")
            }
        }
    };
    match &lang {
        &"kotlin" => {
            bindings::kotlin::run_kotlin_script(target_dir, script_file)?;
        }
        &"python" => {
            bindings::python::run_python_script(target_dir, script_file)?;
        }
        _ => bail!(
            "Somehow tried to launch interpreter for unsupported language {}",
            lang
        ),
    }
    Ok(())
}

// Given the path to a compiled `uniffi` library file, extract and deserialize the
// `ComponentInterface that was stored therein. This returns an error if the file does
// not contain a `ComponentInterface` definition or if it was generated with an incompatible
// version of `uniffi`.
fn get_component_interface_from_library(
    lib_file: &std::ffi::OsStr,
) -> Result<interface::ComponentInterface> {
    use object::read::{Object, ObjectSection};
    let lib_bytes = std::fs::read(lib_file)?;
    let lib = object::read::File::parse(lib_bytes.as_slice())?;
    let idl_section = lib.section_by_name(".uniffi_idl");
    Ok(match idl_section {
        None => bail!("Not a uniffi library: no `.uniffi_idl` section found"),
        Some(idl_section) => match idl_section.uncompressed_data() {
            Err(_) => bail!("Not a uniffi library: missing or corrupt `.uniffi_idl` section"),
            Ok(defn) => interface::ComponentInterface::from_bincode(&defn)?,
        },
    })
}

// Resolve the location of the default library file, relative to the running executable.
// If invoked with a default library file, our command-line tool is running from within the build process
// of a consuming crate. We should therefore look for the file in the build target directory that
// contains the current executable, or failing that, in the current directory.
fn resolve_default_library_file(component_name: &str) -> Result<std::ffi::OsString> {
    let mut lib_path = current_target_dir()?;
    lib_path.push(library_name(component_name));
    Ok(lib_path.into_os_string())
}

fn current_target_dir() -> Result<PathBuf> {
    let curdir = std::env::current_dir()
        .map_err(|_| anyhow!("program has no current directory for some reason"))?;
    Ok(match std::env::current_exe() {
        Err(_) => curdir,
        Ok(exe) => match exe.parent() {
            None => curdir,
            Some(p) => p.to_path_buf(),
        },
    })
}

// XXX TODO: this hard-coding of library file extension probably won't work well
// for cross-compiling (or for windows, sorry :markh...)
#[cfg(any(target_os = "ios", target_os = "macos"))]
fn library_name(component_name: &str) -> String {
    format!("libuniffi_{}.dylib", component_name)
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn library_name(component_name: &str) -> String {
    format!("libuniffi_{}.so", component_name)
}
