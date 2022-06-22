/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use camino::Utf8PathBuf;
use std::env::consts::DLL_EXTENSION;
use std::fs;
use std::process::{Command, Stdio};

// Cache the path to bindings_ir_tests.so to avoid running `cargo build` multiple times.
lazy_static::lazy_static! {
    static ref TARGET_DIRECTORY: Utf8PathBuf = calc_target_dir();
    static ref TEST_SO_PATH: Utf8PathBuf = calc_test_so_path();
}

// Create at test directory
//
// This sets up a new directory inside the testing tempdir, with `bindings_ir_tests.so`
// file inside it.  Bindings renderer tests can then create test scripts, modules, jar files, etc.
// inside this directory, compile everything together, and run the tests.
//
// target_tempdir should be a copy of env!("CARGO_TARGET_TMPDIR").
// name is the name of a subdirectory to create
pub fn setup_test_dir(name: &str) -> Utf8PathBuf {
    let dir = TARGET_DIRECTORY.join("bindings-ir-tests").join(name);
    if dir.exists() {
        // Clean out any files from previous runs
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        TEST_SO_PATH.as_path(),
        dir.join(TEST_SO_PATH.file_name().unwrap()),
    )
    .unwrap();
    dir
}

fn calc_target_dir() -> Utf8PathBuf {
    cargo_metadata::MetadataCommand::new()
        .exec()
        .expect("Error running cargo metadata")
        .target_directory
}

fn calc_test_so_path() -> Utf8PathBuf {
    let mut command = Command::new(env!("CARGO"))
        .arg("build")
        .arg("--tests")
        .arg("--message-format=json")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Error running cargo build");
    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let artifact = cargo_metadata::Message::parse_stream(reader)
        .map(|m| m.expect("Error parsing cargo build messages"))
        .find_map(|message| match message {
            cargo_metadata::Message::CompilerArtifact(artifact) => {
                if artifact.target.name == "bindings-ir-tests" {
                    Some(artifact)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Error finding libbindings_ir_tests.so");
    command.wait().unwrap();

    let cdylib_files: Vec<_> = artifact
        .filenames
        .iter()
        .filter(|nm| matches!(nm.extension(), Some(DLL_EXTENSION)))
        .collect();

    match cdylib_files.len() {
        1 => cdylib_files[0].to_owned(),
        n => panic!("Found {n} cdylib files for bindings-ir-tests"),
    }
}
