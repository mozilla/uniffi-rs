/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    env::consts::DLL_EXTENSION,
    fs,
    process::{Command, Stdio},
};

use anyhow::{bail, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Message, Metadata, MetadataCommand, Package};
use clap::{Parser, Subcommand};

fn main() {
    run_main().unwrap();
}

fn run_main() -> Result<()> {
    let args = Cli::parse();
    let context = Context::new("swift")?;
    match args.command {
        Commands::Print => {
            let out_dir = context.out_dir_base.join("print");
            generate_sources(&context, args.language, &out_dir)?;
            print_sources(&out_dir).unwrap();
        }
        Commands::SaveDiff => {
            let out_dir = context.out_dir_base.join("old");
            generate_sources(&context, args.language, &out_dir)?;
        }
        Commands::Diff => {
            let out_dir = context.out_dir_base.join("new");
            generate_sources(&context, args.language, &out_dir)?;
            diff_sources(&context.out_dir_base)?;
        }
    };
    Ok(())
}

/// Scaffolding and bindings generator for Rust
#[derive(Parser)]
#[clap(name = "uniffi-bindgen")]
#[clap(version = clap::crate_version!())]
#[clap(propagate_version = true)]
struct Cli {
    language: TargetLanguage,
    #[clap(subcommand)]
    command: Commands,
}

/// Enumeration of all foreign language targets currently supported by our CLI.
///
#[derive(Copy, Clone, Eq, PartialEq, Hash, clap::ValueEnum)]
enum TargetLanguage {
    Kotlin,
    Swift,
    Python,
    Ruby,
}

#[derive(Subcommand)]
enum Commands {
    /// Print out the generated source
    Print,
    /// Save the generated source to a target directory for future `diff` commands
    SaveDiff,
    /// Run a diff of the generated sources against the last `save-diff` command
    ///
    /// Usage:
    ///
    /// - cargo run -p test-bindings swift save-diff
    /// - Loop:
    ///   - <make some change to the swift bindings>
    ///   - cargo run -p test-bindings swift diff
    ///   - <inspect the diff for changes>
    Diff,
}

#[derive(Debug)]
struct Context {
    // Name of the crate we're generating source for
    crate_name: String,
    // Path to the cdylib for the crate
    cdylib_path: Utf8PathBuf,
    // Base directory for writing generated files to
    out_dir_base: Utf8PathBuf,
}

impl Context {
    fn new(language: &str) -> Result<Self> {
        let metadata = MetadataCommand::new().exec()?;
        let package = Self::find_current_package(&metadata)?;
        let (crate_name, cdylib_path) = Self::find_crate_and_cdylib(&package)?;
        let out_dir_base = metadata
            .target_directory
            .join("test-bindgen")
            .join(language)
            .join(&crate_name);

        Ok(Self {
            out_dir_base,
            cdylib_path,
            crate_name,
        })
    }

    fn find_current_package(metadata: &Metadata) -> Result<Package> {
        let current_dir = Utf8PathBuf::try_from(std::env::current_dir()?)?;
        for package in &metadata.packages {
            if current_dir.starts_with(package.manifest_path.parent().unwrap()) {
                return Ok(package.clone());
            }
        }
        bail!("Can't determine current package (current_dir: {current_dir})")
    }

    fn find_crate_and_cdylib(package: &Package) -> Result<(String, Utf8PathBuf)> {
        let cdylib_targets = package
            .targets
            .iter()
            .filter(|t| t.crate_types.iter().any(|t| t == "cdylib"))
            .collect::<Vec<_>>();
        let target = match cdylib_targets.len() {
            1 => cdylib_targets[0],
            n => bail!("Found {n} cdylib targets for {}", package.name),
        };

        let mut command = Command::new("cargo")
            .args(["build", "--message-format=json-render-diagnostics"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let reader = std::io::BufReader::new(command.stdout.take().unwrap());
        for message in cargo_metadata::Message::parse_stream(reader) {
            if let Message::CompilerArtifact(artifact) = message? {
                if artifact.target == *target {
                    for filename in artifact.filenames.iter() {
                        if matches!(filename.extension(), Some(DLL_EXTENSION)) {
                            return Ok((target.name.clone(), filename.clone()));
                        }
                    }
                }
            }
        }
        bail!("cdylib not found for crate {}", package.name)
    }
}

fn generate_sources(context: &Context, language: TargetLanguage, dir: &Utf8Path) -> Result<()> {
    let language = match language {
        TargetLanguage::Swift => "swift",
        TargetLanguage::Kotlin => "kotlin",
        TargetLanguage::Python => "python",
        TargetLanguage::Ruby => "ruby",
    };
    let code = Command::new("cargo")
        .args([
            "run",
            "-p",
            "uniffi-bindgen-cli",
            "generate",
            "--language",
            language,
            "--out-dir",
            dir.as_str(),
            "--library",
            context.cdylib_path.as_str(),
            "--crate",
            &context.crate_name,
        ])
        .spawn()?
        .wait()?
        .code();
    match code {
        Some(0) => Ok(()),
        Some(code) => bail!("uniffi-bindgen-cli exited with {code}"),
        None => bail!("uniffi-bindgen-cli terminated by signal"),
    }
}

fn print_sources(dir: &Utf8Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let contents = fs::read_to_string(&path)?;
        println!(
            "-------------------- {} --------------------",
            path.file_name().unwrap().to_string_lossy()
        );
        println!("{contents}");
        println!();
    }
    Ok(())
}

fn diff_sources(out_dir_base: &Utf8Path) -> Result<()> {
    Command::new("diff")
        .args(["-dur", "old", "new", "--color=auto"])
        .current_dir(out_dir_base)
        .spawn()?
        .wait()?;
    Ok(())
}
