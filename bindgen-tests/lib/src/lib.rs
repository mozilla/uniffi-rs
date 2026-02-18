/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

uniffi::setup_scaffolding!("uniffi_bindgen_tests");

#[cfg(feature = "callback_interfaces")]
pub mod callback_interfaces;

#[cfg(feature = "complex_fns")]
pub mod complex_fns;

#[cfg(feature = "compound_types")]
pub mod compound_types;

#[cfg(feature = "custom_types")]
pub mod custom_types;

#[cfg(feature = "enums")]
pub mod enums;

#[cfg(feature = "errors")]
pub mod errors;

#[cfg(feature = "external-types")]
pub mod external_types;

#[cfg(feature = "futures")]
pub mod futures;

#[cfg(feature = "interfaces")]
pub mod interfaces;

#[cfg(feature = "primitive_types")]
pub mod primitive_types;

#[cfg(feature = "records")]
pub mod records;

#[cfg(feature = "renames")]
pub mod renames;

#[cfg(feature = "simple_fns")]
pub mod simple_fns;

#[cfg(feature = "trait_interfaces")]
pub mod trait_interfaces;

// Utility functions for the Rust tests
pub mod test_util {
    use camino::{Utf8Path, Utf8PathBuf};
    use std::{
        env::{
            consts::{DLL_PREFIX, DLL_SUFFIX},
            var,
        },
        fs,
        process::Command,
        sync::OnceLock,
    };

    pub fn current_crate_dir() -> Utf8PathBuf {
        Utf8PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap())
    }

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

    pub fn library_filename() -> String {
        format!("{DLL_PREFIX}uniffi_bindgen_tests{DLL_SUFFIX}")
    }

    pub fn setup_test_dir(language_name: &str) -> Utf8PathBuf {
        let path = workspace_dir()
            .join("target")
            .join("uniffi-tests")
            .join(language_name);
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    pub fn build_library(tempdir: &Utf8Path) {
        Command::new("cargo")
            .args(["build", "-p", "uniffi-bindgen-tests"])
            .status()
            .unwrap();
        let target_dir = workspace_dir().join("target/debug").to_string();
        let library_filename = library_filename();
        fs::copy(
            format!("{target_dir}/{library_filename}"),
            tempdir.join(&library_filename),
        )
        .unwrap();
    }

    pub fn generate_sources(tempdir: &Utf8Path, language: uniffi::TargetLanguage) {
        let library_filename = library_filename();
        uniffi::generate(uniffi::GenerateOptions {
            languages: vec![language],
            source: tempdir.join(library_filename),
            format: false,
            out_dir: tempdir.to_path_buf(),
            ..uniffi::GenerateOptions::default()
        })
        .unwrap();
    }

    // Copy sources to the temp directory
    //
    // globspec is relative to the current crate directory
    pub fn copy_test_sources(tempdir: &Utf8Path, globspec: &str) {
        for path in glob::glob(globspec).unwrap() {
            let path = Utf8PathBuf::from_path_buf(path.unwrap()).unwrap();
            std::fs::create_dir_all(tempdir.join(path.parent().unwrap())).unwrap();
            fs::copy(&path, tempdir.join(&path)).unwrap();
        }
    }
}
