/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Args, Parser};

use uniffi_bindgen::bindings::{generate_swift_bindings, SwiftBindingsOptions};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    kinds: Kinds,
    #[clap(name = "PATH_TO_LIBRARY_OR_UDL")]
    /// UDL File / path to generate bindings for
    source: Utf8PathBuf,
    /// Directory to generate files in
    out_dir: Utf8PathBuf,
    /// Generate a XCFramework-compatible modulemap
    #[arg(long)]
    xcframework: bool,
    /// module name for the generated modulemap
    #[arg(long)]
    module_name: Option<String>,
    /// filename for the generate modulemap
    #[arg(long)]
    modulemap_filename: Option<String>,
    /// Whether we should exclude dependencies when running "cargo metadata".
    /// This will mean external types may not be resolved if they are implemented in crates
    /// outside of this workspace.
    /// This can be used in environments when all types are in the namespace and fetching
    /// all sub-dependencies causes obscure platform specific problems.
    #[clap(long)]
    metadata_no_deps: bool,
    /// What frameworks to link against when generating the modulemap file.
    #[arg(long)]
    link_frameworks: Vec<String>,
    /// Path to optional uniffi config file. This config is merged with the `uniffi.toml` config present in each crate, with its values taking precedence.
    #[clap(long, short)]
    config: Option<Utf8PathBuf>,

    /// Path to optional crate config file
    #[clap(long)]
    crate_metadata: Option<Utf8PathBuf>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = true)]
struct Kinds {
    /// Generate swift files
    #[arg(long)]
    swift_sources: bool,

    /// Generate header files
    #[arg(long)]
    headers: bool,

    /// Generate modulemap
    #[arg(long)]
    modulemap: bool,
}

pub fn run_main() -> Result<()> {
    let cli = Cli::parse();
    generate_swift_bindings(cli.into())
}

impl From<Cli> for SwiftBindingsOptions {
    fn from(cli: Cli) -> Self {
        Self {
            generate_swift_sources: cli.kinds.swift_sources,
            generate_headers: cli.kinds.headers,
            generate_modulemap: cli.kinds.modulemap,
            source: cli.source,
            out_dir: cli.out_dir,
            xcframework: cli.xcframework,
            module_name: cli.module_name,
            modulemap_filename: cli.modulemap_filename,
            metadata_no_deps: cli.metadata_no_deps,
            link_frameworks: cli.link_frameworks,
            config_override: cli.config,
            crate_metadata: cli.crate_metadata,
        }
    }
}
