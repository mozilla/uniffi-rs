use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use clap::{command, Parser};

// TODO: Add all-features and no-default-features options
// TODO: Add univeral library override option(s)

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Rust package to use for building the Swift package
    ///
    /// Cargo metadata will be searched to determine the library name.
    #[arg(short = 'p', long, value_name = "SPEC")]
    package_name: String,

    /// Type of library to package, defaults to staticlib.
    #[arg(long, value_name = "LIB_TYPE")]
    library_type: Option<LibraryType>,

    /// Directory in which to generate the Swift package, i.e., Package.swift parent dir
    #[arg(short = 'o', long)]
    out_dir: Utf8PathBuf,

    /// Swift package name
    ///
    /// Defaults to the package library name.
    #[arg(long, value_name = "NAME")]
    swift_package_name: Option<String>,

    /// Path to manifest for Rust workspace/package
    ///
    /// Defaults to search from current working path
    #[arg(long, value_name = "MANIFEST_PATH")]
    manifest_path: Option<Utf8PathBuf>,

    /// Consolidate crate bindings into single Swift target.
    ///
    /// Otherwise separate Swift targets will be generated
    #[arg(short = 'c', long)]
    consolidate: bool,

    /// Builds package for specified targets.
    ///
    /// Otherwise assumes all targets have been built in the default target dir.
    #[arg(short = 'b', long)]
    build: bool,

    /// Build artifacts in release mode, with optimization
    ///
    /// Requires build flag to be set
    #[arg(short = 'r', long)]
    release: bool,

    /// Space or comma separated list of features to activate
    ///
    /// Requires build flag to be set
    #[arg(short = 'F', long)]
    features: Vec<String>,

    /// Target for target triple to include
    #[arg(long)]
    target: Vec<String>,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum LibraryType {
    /// Build an embedded XCFramework with static libraries
    Staticlib,
    /// Build an embedded XCFrameowrk with embedded dynamic Framework libraries
    Dylib,
}

pub fn run_main() -> Result<()> {
    let _ = Cli::parse();

    // TODO: Specify targets from command line with default.
    let _targets = [
        "x86_64-apple-ios",
        "aarch64-apple-ios-sim",
        "aarch64-apple-ios",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
    ];

    // TODO: Override xcframework libraries from command line.
    let _library_targets = [
        vec!["x86_64-apple-ios", "aarch64-apple-ios-sim"], // iOS simulator
        vec!["aarch64-apple-ios"],                         // iOS
        vec!["x86_64-apple-darwin", "aarch64-apple-darwin"], // macOS
    ];

    bail!("Building a Swift Package is unimplemented!");
}
