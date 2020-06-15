/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::prelude::*;
use std::{
    env,
    collections::HashMap,
    convert::TryFrom, convert::TryInto,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::bail;
use anyhow::Result;
use anyhow::anyhow;

pub mod interface;
pub mod scaffolding;
pub mod bindings;
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
    let idl = slurp_file(idl_file).map_err(|_| anyhow::anyhow!("Failed to read IDL from {}", &idl_file))?;
    let component= idl.parse::<interface::ComponentInterface>().map_err(|e| anyhow::anyhow!("Failed to parse IDL: {}", e))?;
    let mut filename = Path::new(idl_file).file_stem().ok_or_else(|| anyhow!("not a file"))?.to_os_string();
    filename.push(".uniffi.rs");
    let mut out_file = PathBuf::from(env::var("OUT_DIR").map_err(|_| anyhow::anyhow!("No $OUT_DIR specified"))?);
    out_file.push(filename);
    let mut f = File::create(out_file).map_err(|e| anyhow!("Failed to create output file: {:?}", e))?;
    write!(f, "{}", RustScaffolding::new(&component)).map_err(|e| anyhow!("Failed to write output file: {:?}", e))
}

// Call this when generating forgein language bindings from the command-line.
// It implements its own argument parsing, but you must specify the IDL file
// contents as a string literal; this is to encourage consumers to embed the
// contents of the IDL file in the executable, and guard against generating
// bindings using a different IDL file or a different version of uniffi than
// was used to generate the underlying component.

pub fn run_bindings_helper(idl: &str) -> Result<()> {
    let curdir = std::env::current_dir().map_err(|_| anyhow!("program has no current directory for some reason"))?;
    let default_target_dir = match std::env::current_exe() {
        Err(_) => curdir,
        Ok(exe) => match exe.parent() {
            None => curdir,
            Some(p) => p.to_path_buf(),
        }
    };
    // TODO: handle some nice command-line options for different languages.
    // For now we just always compile .kt bindings into a .jar file.
    let app = clap::App::new("uniffi")
        .about("Foreign language bindings generator for Rust")
        .arg(
            clap::Arg::with_name("target_dir")
                .takes_value(true)
                .long("--target-dir")
                .default_value(default_target_dir.to_str().ok_or_else(|| anyhow!("invalid default directory"))?)
                .help("Path to directory into which to write output file(s)")
        )
        .subcommand(clap::SubCommand::with_name("generate")
                        .about("Generate foreign language bindings (currently only for kotlin)")
        )
        .subcommand(clap::SubCommand::with_name("exec")
                        .about("Execute foreign language code with component bindings (currently only for kotlin)")
                        .arg(
                            clap::Arg::with_name("script")
                                .takes_value(true)
                                .help("files to execute")
                        )
        );

    let matches = app.get_matches();
    match matches.subcommand() {
        ("generate", Some(m)) => {
            println!("Parsing IDL...");
            let ci= idl.parse::<interface::ComponentInterface>()?;
            println!("Generating Kotlin bindings...");
            bindings::kotlin::compile_kotlin_bindings(&ci, matches.value_of("target_dir").unwrap())?;
            println!("Done!");
        },
        ("exec", Some(m)) => {
            bindings::kotlin::run_kotlin_script(matches.value_of("target_dir").unwrap(), m.value_of("script"))?;
        },
        _ => println!("No command specified; try `--help` for some help."),
    }
    Ok(())
}