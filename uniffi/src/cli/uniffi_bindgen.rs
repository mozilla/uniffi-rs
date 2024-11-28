/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use uniffi_bindgen::{bindings::*, cli_support};
use uniffi_meta::MetadataGroup;

/// Enumeration of all foreign language targets currently supported by our CLI.
///
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, clap::Subcommand, clap::ValueEnum)]
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

        /// Whether we should exclude dependencies when running "cargo metadata".
        /// This will mean external types may not be resolved if they are implemented in crates
        /// outside of this workspace.
        /// This can be used in environments when all types are in the namespace and fetching
        /// all sub-dependencies causes obscure platform specific problems.
        #[clap(long)]
        metadata_no_deps: bool,
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

    /// Print a stage of the bindings render pipeline
    Peek {
        /// Pass in a cdylib path rather than a UDL file
        #[clap(long = "library")]
        library_mode: bool,

        /// Path to the UDL file, or cdylib if `library-mode` is specified
        source: Utf8PathBuf,

        /// When `--library` is passed, only generate bindings for one crate.
        /// When `--library` is not passed, use this as the crate name instead of attempting to
        /// locate and parse Cargo.toml.
        #[clap(long = "crate")]
        crate_name: Option<String>,

        /// Whether we should exclude dependencies when running "cargo metadata".
        /// This will mean external types may not be resolved if they are implemented in crates
        /// outside of this workspace.
        /// This can be used in environments when all types are in the namespace and fetching
        /// all sub-dependencies causes obscure platform specific problems.
        #[clap(long)]
        metadata_no_deps: bool,

        /// Target to display
        #[clap(subcommand)]
        target: PeekTarget,
    },

    /// Save a stage of the bindings render pipeline for later diffing
    DiffSave {
        /// Pass in a cdylib path rather than a UDL file
        #[clap(long = "library")]
        library_mode: bool,

        /// Path to the UDL file, or cdylib if `library-mode` is specified
        source: Utf8PathBuf,

        /// When `--library` is passed, only generate bindings for one crate.
        /// When `--library` is not passed, use this as the crate name instead of attempting to
        /// locate and parse Cargo.toml.
        #[clap(long = "crate")]
        crate_name: Option<String>,

        /// Whether we should exclude dependencies when running "cargo metadata".
        /// This will mean external types may not be resolved if they are implemented in crates
        /// outside of this workspace.
        /// This can be used in environments when all types are in the namespace and fetching
        /// all sub-dependencies causes obscure platform specific problems.
        #[clap(long)]
        metadata_no_deps: bool,
    },

    /// Diff a stage of the bindings render pipeline against the data last saved with DiffSave
    Diff {
        /// Pass in a cdylib path rather than a UDL file
        #[clap(long = "library")]
        library_mode: bool,

        /// Path to the UDL file, or cdylib if `library-mode` is specified
        source: Utf8PathBuf,

        /// When `--library` is passed, only generate bindings for one crate.
        /// When `--library` is not passed, use this as the crate name instead of attempting to
        /// locate and parse Cargo.toml.
        #[clap(long = "crate")]
        crate_name: Option<String>,

        /// Whether we should exclude dependencies when running "cargo metadata".
        /// This will mean external types may not be resolved if they are implemented in crates
        /// outside of this workspace.
        /// This can be used in environments when all types are in the namespace and fetching
        /// all sub-dependencies causes obscure platform specific problems.
        #[clap(long)]
        metadata_no_deps: bool,

        /// Target to display
        #[clap(subcommand)]
        target: PeekTarget,
    },
}

#[derive(Clone, Debug, clap::Subcommand)]
enum PeekTarget {
    Metadata,
    Ir,
    PythonIr,
    Kotlin,
    Swift,
    Python,
    Ruby,
}

impl PeekTarget {
    fn all() -> Vec<PeekTarget> {
        vec![
            PeekTarget::Metadata,
            PeekTarget::Ir,
            PeekTarget::PythonIr,
            PeekTarget::Kotlin,
            PeekTarget::Swift,
            PeekTarget::Python,
            PeekTarget::Ruby,
        ]
    }
}

impl fmt::Display for PeekTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metadata => write!(f, "metadata"),
            Self::Ir => write!(f, "ir"),
            Self::PythonIr => write!(f, "python-ir"),
            Self::Kotlin => write!(f, "kotlin"),
            Self::Swift => write!(f, "swift"),
            Self::Python => write!(f, "python"),
            Self::Ruby => write!(f, "ruby"),
        }
    }
}

fn config_supplier(
    metadata_no_deps: bool,
) -> Result<impl uniffi_bindgen::BindgenCrateConfigSupplier> {
    #[cfg(feature = "cargo-metadata")]
    {
        use uniffi_bindgen::cargo_metadata::CrateConfigSupplier;
        let mut cmd = cargo_metadata::MetadataCommand::new();
        if metadata_no_deps {
            cmd.no_deps();
        }
        let metadata = cmd.exec().context("error running cargo metadata")?;
        Ok(CrateConfigSupplier::from(metadata))
    }
    #[cfg(not(feature = "cargo-metadata"))]
    Ok(Auniffi_bindgen::EmptyCrateConfigSupplier)
}

fn gen_library_mode(
    library_path: &camino::Utf8Path,
    crate_name: Option<String>,
    languages: Vec<TargetLanguage>,
    cfo: Option<&camino::Utf8Path>,
    out_dir: &camino::Utf8Path,
    fmt: bool,
    metadata_no_deps: bool,
) -> anyhow::Result<()> {
    use uniffi_bindgen::library_mode::generate_bindings;

    let config_supplier = config_supplier(metadata_no_deps)?;

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
                &config_supplier,
                cfo,
                out_dir,
                fmt,
            )?
            .len(),
            TargetLanguage::Python => generate_bindings(
                library_path,
                crate_name.clone(),
                &PythonBindingGenerator,
                &config_supplier,
                cfo,
                out_dir,
                fmt,
            )?
            .len(),
            TargetLanguage::Ruby => generate_bindings(
                library_path,
                crate_name.clone(),
                &RubyBindingGenerator,
                &config_supplier,
                cfo,
                out_dir,
                fmt,
            )?
            .len(),
            TargetLanguage::Swift => generate_bindings(
                library_path,
                crate_name.clone(),
                &SwiftBindingGenerator,
                &config_supplier,
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
            metadata_no_deps,
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
                    metadata_no_deps,
                )?;
            } else {
                if metadata_no_deps {
                    panic!("--metadata-no-deps makes no sense when not in library mode")
                }
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
        Commands::Peek {
            crate_name,
            source,
            library_mode,
            metadata_no_deps,
            target,
        } => {
            let config_supplier = config_supplier(metadata_no_deps)?;
            let (metadata, cdylib) = if library_mode {
                let metadata = uniffi_bindgen::load_metadata_from_library(
                    &source,
                    crate_name.as_deref(),
                    config_supplier,
                )?;
                (metadata, Some(source.to_string()))
            } else {
                let metadata =
                    uniffi_bindgen::load_metadata_from_udl(&source, crate_name.as_deref())?;
                (metadata, None)
            };

            cli_support::peek(get_peek_items(&target, metadata, cdylib, metadata_no_deps)?);
        }
        Commands::DiffSave {
            crate_name,
            source,
            library_mode,
            metadata_no_deps,
        } => {
            let config_supplier = config_supplier(metadata_no_deps)?;
            let (metadata, cdylib) = if library_mode {
                let metadata = uniffi_bindgen::load_metadata_from_library(
                    &source,
                    crate_name.as_deref(),
                    config_supplier,
                )?;
                (metadata, Some(source.to_string()))
            } else {
                let metadata =
                    uniffi_bindgen::load_metadata_from_udl(&source, crate_name.as_deref())?;
                (metadata, None)
            };

            for target in PeekTarget::all() {
                let diff_dir = cli_support::diff_dir_from_cargo_metadata(&target)?;
                cli_support::save_diff(
                    &diff_dir,
                    get_peek_items(&target, metadata.clone(), cdylib.clone(), metadata_no_deps)?,
                )?
            }
        }
        Commands::Diff {
            crate_name,
            source,
            library_mode,
            metadata_no_deps,
            target,
        } => {
            let config_supplier = config_supplier(metadata_no_deps)?;
            let (metadata, cdylib) = if library_mode {
                let metadata = uniffi_bindgen::load_metadata_from_library(
                    &source,
                    crate_name.as_deref(),
                    config_supplier,
                )?;
                (metadata, Some(source.to_string()))
            } else {
                let metadata =
                    uniffi_bindgen::load_metadata_from_udl(&source, crate_name.as_deref())?;
                (metadata, None)
            };

            let diff_dir = cli_support::diff_dir_from_cargo_metadata(&target)?;
            cli_support::diff(
                &diff_dir,
                get_peek_items(&target, metadata, cdylib, metadata_no_deps)?,
            )?;
        }
    };
    Ok(())
}

fn get_peek_items(
    target: &PeekTarget,
    metadata: Vec<MetadataGroup>,
    cdylib: Option<String>,
    metadata_no_deps: bool,
) -> Result<Vec<(String, String)>> {
    match target {
        PeekTarget::Metadata => metadata
            .into_iter()
            .map(|group| Ok((group.namespace.name.clone(), format!("{group:#?}"))))
            .collect(),
        PeekTarget::Ir => {
            let irs = uniffi_bindgen::metadata_groups_to_irs(metadata)?;
            irs.into_iter()
                .map(|ir| Ok((ir.namespace.clone(), format!("{ir:#?}"))))
                .collect()
        }
        PeekTarget::PythonIr => {
            let irs_and_configs = uniffi_bindgen::metadata_groups_to_irs_and_configs(
                metadata,
                cdylib,
                PythonBindingGenerator,
                config_supplier(metadata_no_deps)?,
            )?;
            irs_and_configs
                .into_iter()
                .map(|(ir, config)| {
                    let ir = PythonBindingsIr::from_general_ir(ir, config)?;
                    Ok((ir.namespace.clone(), format!("{ir:#?}")))
                })
                .collect()
        }
        PeekTarget::Kotlin => {
            let components = uniffi_bindgen::metadata_groups_to_components(
                metadata,
                cdylib,
                KotlinBindingGenerator,
                config_supplier(metadata_no_deps)?,
            )?;
            components
                .into_iter()
                .map(|component| {
                    let name = format!("{}.py", component.ci.namespace());
                    let contents = kotlin::generate_bindings(&component.config, &component.ci)?;
                    Ok((name, contents))
                })
                .collect()
        }
        PeekTarget::Swift => {
            let components = uniffi_bindgen::metadata_groups_to_components(
                metadata,
                cdylib,
                SwiftBindingGenerator,
                config_supplier(metadata_no_deps)?,
            )?;
            let mut all_content = vec![(
                "module.modulemap".to_string(),
                swift::generate_modulemap(
                    "module".to_string(),
                    components
                        .iter()
                        .map(|c| format!("{}.h", c.ci.namespace()))
                        .collect(),
                    false,
                )?,
            )];
            for component in components {
                all_content.push((
                    format!("{}.h", component.ci.namespace()),
                    swift::generate_header(&component.config, &component.ci)?,
                ));
                all_content.push((
                    format!("{}.swift", component.ci.namespace()),
                    swift::generate_swift(&component.config, &component.ci)?,
                ));
            }
            Ok(all_content)
        }
        PeekTarget::Python => {
            let irs_and_configs = uniffi_bindgen::metadata_groups_to_irs_and_configs(
                metadata,
                cdylib,
                PythonBindingGenerator,
                config_supplier(metadata_no_deps)?,
            )?;
            irs_and_configs
                .into_iter()
                .map(|(ir, config)| {
                    let ir = PythonBindingsIr::from_general_ir(ir, config)?;
                    let name = format!("{}.py", ir.namespace);
                    let contents = python::generate_python_bindings_from_ir(ir)?;
                    Ok((name, contents))
                })
                .collect()
        }
        PeekTarget::Ruby => {
            let components = uniffi_bindgen::metadata_groups_to_components(
                metadata,
                cdylib,
                RubyBindingGenerator,
                config_supplier(metadata_no_deps)?,
            )?;
            components
                .into_iter()
                .map(|component| {
                    let name = format!("{}.py", component.ci.namespace());
                    let contents = ruby::generate_ruby_bindings(&component.config, &component.ci)?;
                    Ok((name, contents))
                })
                .collect()
        }
    }
}
