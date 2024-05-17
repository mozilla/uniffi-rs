/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use std::fmt;
use uniffi_bindgen::bindings::*;

/// Enumeration of all foreign language targets currently supported by our CLI.
///
#[derive(Copy, Clone, Eq, PartialEq, Hash, clap::ValueEnum)]
enum TargetLanguage {
    Kotlin,
    Swift,
    Python,
    Ruby,
}

impl fmt::Display for TargetLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kotlin => write!(f, "kotlin"),
            Self::Swift => write!(f, "swift"),
            Self::Python => write!(f, "python"),
            Self::Ruby => write!(f, "ruby"),
        }
    }
}

impl TryFrom<&str> for TargetLanguage {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "kotlin" | "kt" | "kts" => TargetLanguage::Kotlin,
            "swift" => TargetLanguage::Swift,
            "python" | "py" => TargetLanguage::Python,
            "ruby" | "rb" => TargetLanguage::Ruby,
            _ => bail!("Unknown or unsupported target language: \"{value}\""),
        })
    }
}

impl TryFrom<&std::ffi::OsStr> for TargetLanguage {
    type Error = anyhow::Error;
    fn try_from(value: &std::ffi::OsStr) -> Result<Self> {
        match value.to_str() {
            None => bail!("Unreadable target language"),
            Some(s) => s.try_into(),
        }
    }
}

impl TryFrom<String> for TargetLanguage {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        TryFrom::try_from(value.as_str())
    }
}

// Structs to help our cmdline parsing. Note that docstrings below form part
// of the "help" output.

/// Scaffolding and bindings generator for Rust
#[derive(Parser)]
#[clap(name = "uniffi-bindgen")]
#[clap(version = clap::crate_version!())]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate foreign language bindings
    Generate {
        /// Foreign language(s) for which to build bindings.
        #[clap(long, short, value_enum)]
        language: Vec<TargetLanguage>,

        /// Directory in which to write generated files. Default is same folder as .udl file.
        #[clap(long, short)]
        out_dir: Option<Utf8PathBuf>,

        /// Do not try to format the generated bindings.
        #[clap(long, short)]
        no_format: bool,

        /// Path to optional uniffi config file. This config is merged with the `uniffi.toml` config present in each crate, with its values taking precedence.
        #[clap(long, short)]
        config: Option<Utf8PathBuf>,

        /// Extract proc-macro metadata from a native lib (cdylib or staticlib) for this crate.
        #[clap(long)]
        lib_file: Option<Utf8PathBuf>,

        /// Pass in a cdylib path rather than a UDL file
        #[clap(long = "library")]
        library_mode: bool,

        /// When `--library` is passed, only generate bindings for one crate.
        /// When `--library` is not passed, use this as the crate name instead of attempting to
        /// locate and parse Cargo.toml.
        #[clap(long = "crate")]
        crate_name: Option<String>,

        /// Path to the UDL file, or cdylib if `library-mode` is specified
        source: Utf8PathBuf,
    },

    /// Generate Rust scaffolding code
    Scaffolding {
        /// Directory in which to write generated files. Default is same folder as .udl file.
        #[clap(long, short)]
        out_dir: Option<Utf8PathBuf>,

        /// Do not try to format the generated bindings.
        #[clap(long, short)]
        no_format: bool,

        /// Path to the UDL file.
        udl_file: Utf8PathBuf,
    },

    /// Print a debug representation of the interface from a dynamic library
    PrintRepr {
        /// Path to the library file (.so, .dll, .dylib, or .a)
        path: Utf8PathBuf,
    },
}

fn gen_library_mode(
    library_path: &camino::Utf8Path,
    crate_name: Option<String>,
    languages: Vec<TargetLanguage>,
    cfo: Option<&camino::Utf8Path>,
    out_dir: &camino::Utf8Path,
    fmt: bool,
) -> anyhow::Result<()> {
    use uniffi_bindgen::library_mode::generate_bindings;
    for language in languages {
        // to help avoid mistakes we check the library is actually a cdylib, except
        // for swift where static libs are often used to extract the metadata.
        if !matches!(language, TargetLanguage::Swift) && !uniffi_bindgen::is_cdylib(library_path) {
            anyhow::bail!(
                "Generate bindings for {language} requires a cdylib, but {library_path} was given"
            );
        }

        // Type-bounds on trait implementations makes selecting between languages a bit tedious.
        match language {
            TargetLanguage::Kotlin => generate_bindings(
                library_path,
                crate_name.clone(),
                &KotlinBindingGenerator,
                cfo,
                out_dir,
                fmt,
            )?
            .len(),
            TargetLanguage::Python => generate_bindings(
                library_path,
                crate_name.clone(),
                &PythonBindingGenerator,
                cfo,
                out_dir,
                fmt,
            )?
            .len(),
            TargetLanguage::Ruby => generate_bindings(
                library_path,
                crate_name.clone(),
                &RubyBindingGenerator,
                cfo,
                out_dir,
                fmt,
            )?
            .len(),
            TargetLanguage::Swift => generate_bindings(
                library_path,
                crate_name.clone(),
                &SwiftBindingGenerator,
                cfo,
                out_dir,
                fmt,
            )?
            .len(),
        };
    }
    Ok(())
}

fn gen_bindings(
    udl_file: &camino::Utf8Path,
    cfo: Option<&camino::Utf8Path>,
    languages: Vec<TargetLanguage>,
    odo: Option<&camino::Utf8Path>,
    library_file: Option<&camino::Utf8Path>,
    crate_name: Option<&str>,
    fmt: bool,
) -> anyhow::Result<()> {
    use uniffi_bindgen::generate_bindings;
    for language in languages {
        match language {
            TargetLanguage::Kotlin => generate_bindings(
                udl_file,
                cfo,
                KotlinBindingGenerator,
                odo,
                library_file,
                crate_name,
                fmt,
            )?,
            TargetLanguage::Python => generate_bindings(
                udl_file,
                cfo,
                PythonBindingGenerator,
                odo,
                library_file,
                crate_name,
                fmt,
            )?,
            TargetLanguage::Ruby => generate_bindings(
                udl_file,
                cfo,
                RubyBindingGenerator,
                odo,
                library_file,
                crate_name,
                fmt,
            )?,
            TargetLanguage::Swift => generate_bindings(
                udl_file,
                cfo,
                SwiftBindingGenerator,
                odo,
                library_file,
                crate_name,
                fmt,
            )?,
        };
    }
    Ok(())
}

pub fn run_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate {
            language,
            out_dir,
            no_format,
            config,
            lib_file,
            source,
            crate_name,
            library_mode,
        } => {
            if library_mode {
                if lib_file.is_some() {
                    panic!("--lib-file is not compatible with --library.")
                }
                let out_dir = out_dir.expect("--out-dir is required when using --library");
                if language.is_empty() {
                    panic!("please specify at least one language with --language")
                }
                gen_library_mode(
                    &source,
                    crate_name,
                    language,
                    config.as_deref(),
                    &out_dir,
                    !no_format,
                )?;
            } else {
                gen_bindings(
                    &source,
                    config.as_deref(),
                    language,
                    out_dir.as_deref(),
                    lib_file.as_deref(),
                    crate_name.as_deref(),
                    !no_format,
                )?;
            }
        }
        Commands::Scaffolding {
            out_dir,
            no_format,
            udl_file,
        } => {
            uniffi_bindgen::generate_component_scaffolding(
                &udl_file,
                out_dir.as_deref(),
                !no_format,
            )?;
        }
        Commands::PrintRepr { path } => {
            uniffi_bindgen::print_repr(&path)?;
        }
    };
    Ok(())
}
