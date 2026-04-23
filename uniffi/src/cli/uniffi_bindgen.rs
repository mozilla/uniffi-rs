/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;
use uniffi_bindgen::{
    bindings::{generate, python, GenerateOptions, TargetLanguage},
    pipeline::initial,
};
use uniffi_pipeline::PrintOptions;

/// TargetLanguage uniffi_bindgen, with a `clap::ValueEnum` derive.
#[derive(Copy, Clone, ValueEnum)]
enum TargetLanguageArg {
    Kotlin,
    Swift,
    Python,
    Ruby,
}

impl fmt::Display for TargetLanguageArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kotlin => write!(f, "kotlin"),
            Self::Swift => write!(f, "swift"),
            Self::Python => write!(f, "python"),
            Self::Ruby => write!(f, "ruby"),
        }
    }
}

impl From<TargetLanguageArg> for TargetLanguage {
    fn from(l: TargetLanguageArg) -> Self {
        match l {
            TargetLanguageArg::Kotlin => Self::Kotlin,
            TargetLanguageArg::Swift => Self::Swift,
            TargetLanguageArg::Python => Self::Python,
            TargetLanguageArg::Ruby => Self::Ruby,
        }
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
        language: Vec<TargetLanguageArg>,

        /// Directory in which to write generated files. Default is same folder as .udl file.
        #[clap(long, short)]
        out_dir: Option<Utf8PathBuf>,

        /// Do not try to format the generated bindings.
        #[clap(long, short)]
        no_format: bool,

        /// Path to optional uniffi config file. This config is merged with the `uniffi.toml` config present in each crate, with its values taking precedence.
        #[clap(long, short)]
        config: Option<Utf8PathBuf>,

        /// Path to optional crate config file
        #[clap(long)]
        crate_metadata: Option<Utf8PathBuf>,

        /// Deprecated
        ///
        /// This used to signal that a source file is a library rather than a UDL file.
        /// Nowadays, UniFFI will auto-detect this.
        #[clap(long = "library")]
        _library_mode: bool,

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

    /// Inspect the bindings render pipeline
    Pipeline(PipelineArgs),
}

#[derive(Args)]
struct PipelineArgs {
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

    /// Bindings Language
    language: TargetLanguageArg,

    /// Only show passes that match <PASS>
    ///
    /// Use `last` to only show the last pass, this can be useful when you're writing new pipelines
    #[clap(short, long)]
    pass: Option<String>,

    /// Don't show diffs for middle passes
    #[clap(long)]
    no_diff: bool,

    /// Only show data for types with name <FILTER_TYPE>
    #[clap(short = 't', long = "type")]
    filter_type: Option<String>,

    /// Only show data for items with fields that match <FILTER>
    #[clap(short = 'n', long = "name")]
    filter_name: Option<String>,
}

pub fn run_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate {
            language,
            out_dir,
            no_format,
            config,
            crate_metadata,
            source,
            crate_name,
            metadata_no_deps,
            ..
        } => {
            if language.is_empty() {
                panic!("please specify at least one language with --language")
            }

            generate(GenerateOptions {
                languages: language.into_iter().map(TargetLanguage::from).collect(),
                out_dir: out_dir
                    .expect("--out-dir is required when generating {language} bindings"),
                source,
                config_override: config,
                crate_metadata,
                crate_filter: crate_name,
                metadata_no_deps,
                format: !no_format,
            })?;
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
        Commands::Pipeline(args) => {
            let mut paths = uniffi_bindgen::BindgenPaths::default();
            #[cfg(feature = "cargo-metadata")]
            paths.add_cargo_metadata_layer(args.metadata_no_deps)?;

            let initial_root = if args.library_mode {
                initial::Root::from_library(paths, &args.source, args.crate_name)?
            } else {
                initial::Root::from_udl(paths, &args.source, args.crate_name)?
            };

            let opts = PrintOptions {
                pass: args.pass,
                no_diff: args.no_diff,
                filter_type: args.filter_type,
                filter_name: args.filter_name,
            };
            match args.language {
                TargetLanguageArg::Python => python::pipeline().print_passes(initial_root, opts)?,
                language => unimplemented!("{language} does not use the bindings IR pipeline yet"),
            };
        }
    };
    Ok(())
}
