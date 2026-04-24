/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};

// Structs to help our cmdline parsing. Note that docstrings below form part
// of the "help" output.

/// Scaffolding and bindings generator for Rust
#[derive(Parser)]
#[clap(name = "uniffi-bindgen")]
#[clap(version = clap::crate_version!())]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate foreign language bindings
    Bindings {
        #[clap(flatten)]
        args: StandardArgs,

        /// Directory in which to write generated files.
        out_dir: Utf8PathBuf,
    },

    /// Generate Rust scaffolding code
    Scaffolding {
        #[clap(flatten)]
        args: StandardArgs,

        /// Directory in which to write generated files.
        out_dir: Utf8PathBuf,
    },

    /// Inspect the bindings render pipeline
    Pipeline {
        #[clap(flatten)]
        args: StandardArgs,

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
    },
}

#[derive(Args, Default)]
pub struct StandardArgs {
    /// Source to generate bindings from.
    ///
    /// Possible values:
    ///
    /// * Path to a UDL file
    /// * Path to a library file
    /// * `src:[crate-name]` to generate from Rust sources
    pub source: Utf8PathBuf,

    /// Limit binding/scaffolding generation to a single crate.
    #[clap(long = "crate")]
    pub crate_filter: Option<String>,

    /// Path to optional uniffi config file. This config is merged with the `uniffi.toml` config present in each crate, with its values taking precedence.
    #[clap(long, short)]
    pub config: Option<Utf8PathBuf>,

    /// Whether we should exclude dependencies when running "cargo metadata".
    /// This will mean external types may not be resolved if they are implemented in crates
    /// outside of this workspace.
    /// This can be used in environments when all types are in the namespace and fetching
    /// all sub-dependencies causes obscure platform specific problems.
    #[clap(long)]
    pub metadata_no_deps: bool,

    /// Features to enable when generating from Rust sources
    #[clap(short, long)]
    pub features: Vec<String>,

    /// Enable all features
    #[clap(long)]
    pub all_features: bool,

    /// Don't auto-enable default features
    #[clap(long)]
    pub no_default_features: bool,

    /// Target triple to use when generating from Rust sources
    #[clap(long)]
    pub target: Option<String>,
}
