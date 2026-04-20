/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings;
/// Utility functions for running tests/benchmarks
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    env::consts::{DLL_PREFIX, DLL_SUFFIX},
    fs,
    process::Command,
    sync::OnceLock,
};

/// Get the workspace root dir
pub fn workspace_dir() -> Utf8PathBuf {
    static DIR: OnceLock<Utf8PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let output: String = Command::new("cargo")
            .args(["locate-project", "--workspace", "--message-format=plain"])
            .output()
            .unwrap()
            .stdout
            .try_into()
            .unwrap();
        Utf8PathBuf::from(output).parent().unwrap().to_path_buf()
    })
    .to_path_buf()
}

fn library_filename(library_name: &str) -> String {
    let lib_name = library_name.replace("-", "_");
    format!("{DLL_PREFIX}{lib_name}{DLL_SUFFIX}")
}

/// Create a scratch directory to use for testing/benchmarks
pub fn setup_test_dir(name: &str) -> Utf8PathBuf {
    let path = workspace_dir()
        .join("target")
        .join("uniffi-tests")
        .join(name);
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    path
}

/// Build a cdylib library and copy it to the temp directory
pub fn build_library(tempdir: &Utf8Path, package_name: &str, opt: LibraryOptions) {
    let mut extra_args = vec![];
    if !opt.features.is_empty() {
        extra_args.push(format!("--features={}", opt.features.join(",")));
    }
    if opt.no_default_features {
        extra_args.push("--no-default-features".into());
    }

    let status = Command::new("cargo")
        .args(["build", "-p", package_name])
        .args(extra_args)
        .status()
        .unwrap();
    if !status.success() {
        panic!("cargo build -p {package_name} failed");
    }
    let target_dir = workspace_dir().join("target/debug");
    let library_filename = library_filename(opt.library_name.as_deref().unwrap_or(package_name));
    let source = target_dir.join(&library_filename);
    if !source.exists() {
        panic!("build_library: {source} not found");
    }
    fs::copy(source, tempdir.join(&library_filename)).unwrap();
}

#[derive(Default)]
pub struct LibraryOptions {
    /// Name of the cdylib library, if not present then the package name is used
    pub library_name: Option<String>,
    pub features: Vec<String>,
    pub no_default_features: bool,
}

/// Run `uniffi-bindgen generate` and output to the temp directory
pub fn generate_sources(tempdir: &Utf8Path, language: bindings::TargetLanguage) {
    let source = match language {
        bindings::TargetLanguage::Kotlin => "src:uniffi-bindgen-tests-kotlin",
        bindings::TargetLanguage::Python => "src:uniffi-bindgen-tests-python",
        bindings::TargetLanguage::Swift => "src:uniffi-bindgen-tests-swift",
        bindings::TargetLanguage::Ruby => unimplemented!("Ruby tests"),
    };

    bindings::generate(bindings::GenerateOptions {
        languages: vec![language],
        source: source.into(),
        format: false,
        out_dir: tempdir.to_path_buf(),
        ..bindings::GenerateOptions::default()
    })
    .unwrap();
}

// Copy sources to the temp directory
//
// globspec is relative to the current crate directory
pub fn copy_test_sources(tempdir: &Utf8Path, globspec: &str) {
    for path in glob::glob(globspec).unwrap() {
        let path = Utf8PathBuf::from_path_buf(path.unwrap()).unwrap();
        fs::create_dir_all(tempdir.join(path.parent().unwrap())).unwrap();
        fs::copy(&path, tempdir.join(&path)).unwrap();
    }
}
