/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod bindings;
mod cli;
mod config;
pub mod pipeline;
pub mod scaffolding;
#[cfg(feature = "test-util")]
pub mod test_util;

use std::env;

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use cli::{Cli, Commands};
pub use config::Config;
use uniffi_bindgen::BindgenLoader;
use uniffi_pipeline::PrintOptions;

pub use anyhow::Result;

pub fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Bindings { args, out_dir } => {
            generate_bindings_from_args(args, &out_dir)?;
        }
        Commands::Scaffolding { args, out_dir } => {
            generate_bindings_from_args(args, &out_dir)?;
        }
        Commands::Pipeline {
            args,
            pass,
            no_diff,
            filter_type,
            filter_name,
        } => {
            let initial_root = load_initial_root(&args)?;
            let opts = PrintOptions {
                pass,
                no_diff,
                filter_type,
                filter_name,
            };
            pipeline::pipeline().print_passes(initial_root, opts)?
        }
    };
    Ok(())
}

fn load_initial_root(args: &cli::StandardArgs) -> Result<pipeline::InitialRoot> {
    let mut paths = uniffi_bindgen::BindgenPaths::default();
    #[cfg(feature = "cargo-metadata")]
    paths.add_cargo_metadata_layer(uniffi_bindgen::CargoMetadataOptions {
        no_deps: args.metadata_no_deps,
        no_default_features: args.no_default_features,
        all_features: args.all_features,
        features: args.features.clone(),
        target: args.target.clone(),
    })?;

    let loader = BindgenLoader::new(paths);
    let metadata = loader.load_metadata(&args.source)?;
    loader.load_pipeline_initial_root(&args.source, metadata)
}

pub fn generate_bindings(source: &str, out_dir: &Utf8Path) -> Result<()> {
    let args = cli::StandardArgs {
        source: source.into(),
        ..cli::StandardArgs::default()
    };
    generate_bindings_from_args(args, out_dir)
}

pub fn generate_bindings_from_args(args: cli::StandardArgs, out_dir: &Utf8Path) -> Result<()> {
    let initial_root = load_initial_root(&args)?;
    bindings::generate(initial_root, out_dir, args.crate_filter.clone())
}

/// Generate the scaffolding for a `build.rs` script
///
/// This must be run from a `build.rs` script.
/// Generate code is written to `$OUT_DIR/uniffi-bindgen-kotlin-jni/[crate_name].rs`
pub fn generate_scaffolding() {
    _generate_scaffolding().expect("Error while generating scaffolding");
}

fn _generate_scaffolding() -> Result<()> {
    let pkg_name = env::var("CARGO_PKG_NAME").context("CARGO_PKG_NAME env not set")?;
    let features = env::var("CARGO_CFG_FEATURE").context("CARGO_CFG_FEATURE env not set")?;
    let target = env::var("TARGET").context("TARGET env not set")?;
    let out_dir = env::var("OUT_DIR").context("OUT_DIR env not set")?;

    let args = cli::StandardArgs {
        source: Utf8PathBuf::from(format!("src:{pkg_name}")),
        features: features.split(',').map(str::to_string).collect(),
        all_features: false,
        no_default_features: true,
        target: Some(target.to_string()),
        crate_filter: None,
        config: None,
        metadata_no_deps: false,
    };
    let initial_root = load_initial_root(&args)?;
    scaffolding::generate(initial_root, pkg_name, &Utf8PathBuf::from(out_dir))
}
