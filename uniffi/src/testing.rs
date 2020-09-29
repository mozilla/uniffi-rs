/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Runtime support functionality for testing generated bindings.
//!
//! This module helps you run a foreign language script as a testcase to exercise the
//! bindings generated from your rust code. You probably don't want to use it directly,
//! and should instead use the `build_foreign_language_testcases!` macro provided by
//! the `uniffi_macros` crate.

use anyhow::{bail, Result};
use cargo_metadata::Message;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
    sync::Mutex,
};

// These statics are used for a bit of simple caching and concurrency control.
// They map uniffi component crate directories to data about build steps that have already
// been executed by this process.
lazy_static! {
    static ref COMPILED_COMPONENTS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    // Since uniffi-bindgen does the actual generating/compiling of bindings and script files,
    // we ensure that only one call happens at once (making tests pretty much serialized sorry :/).
    static ref UNIFFI_BINDGEN: Mutex<i32> = Mutex::new(0);
}

/// Execute the given foreign-language script as part of a rust test suite.
///
/// This function takes the top-level directory of a uniffi component crate, and the path to
/// a foreign-language test file that exercises that component's bindings. It ensures that the
/// component is compiled and available for use and then executes the foreign language script,
/// returning successfully iff the script exits successfully.
pub fn run_foreign_language_testcase(pkg_dir: &str, idl_file: &str, test_file: &str) -> Result<()> {
    let cdylib_file = ensure_compiled_cdylib(pkg_dir)?;
    let out_dir = Path::new(cdylib_file.as_str())
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Generated cdylib has no parent directory"))?
        .to_str()
        .unwrap();
    let _lock = UNIFFI_BINDGEN.lock();
    run_uniffi_bindgen_test(out_dir, idl_file, test_file)?;
    Ok(())
}

/// Ensure that a uniffi component crate is compiled and ready for use.
///
/// This function takes the top-level directory of a uniffi component crate, ensures that the
/// component's cdylib is compiled and available for use in generating bindings and running
/// foreign language code.
///
/// Internally, this function does a bit of caching and concurrency management to avoid rebuilding
/// the component for multiple testcases.
pub fn ensure_compiled_cdylib(pkg_dir: &str) -> Result<String> {
    // Have we already compiled this component?
    let mut compiled_components = COMPILED_COMPONENTS.lock().unwrap();
    if let Some(cdylib_file) = compiled_components.get(pkg_dir) {
        return Ok(cdylib_file.to_string());
    }
    // Nope, looks like we'll have to compile it afresh.
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--message-format=json").arg("--lib");
    cmd.current_dir(pkg_dir);
    cmd.stdout(Stdio::piped());
    let mut child = cmd.spawn()?;
    let output = std::io::BufReader::new(child.stdout.take().unwrap());
    // Build the crate, looking for any cdylibs that it might produce.
    let cdylibs = Message::parse_stream(output)
        .filter_map(|message| match message {
            Err(e) => Some(Err(e.into())),
            Ok(Message::CompilerArtifact(artifact)) => {
                if artifact.target.kind.iter().any(|item| item == "cdylib") {
                    Some(Ok(artifact))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Result<Vec<_>>>()?;
    if !child.wait()?.success() {
        bail!("Failed to execute `cargo build`");
    }
    // If we didn't just build exactly one cdylib, we're going to have a bad time.
    match cdylibs.len() {
        0 => bail!("Crate did not produce any cdylibs, it must not be a uniffi component"),
        1 => (),
        _ => bail!("Crate produced multiple cdylibs, it must not be a uniffi component"),
    }
    let cdylib_files: Vec<_> = cdylibs[0]
        .filenames
        .iter()
        .filter(|nm| match nm.extension().unwrap_or_default().to_str() {
            Some("dylib") | Some("so") => true,
            _ => false,
        })
        .collect();
    if cdylib_files.len() != 1 {
        bail!("Failed to build exactly one cdylib file, it must not be a uniffi component");
    }
    let cdylib_file = cdylib_files[0].to_string_lossy().into_owned();
    // Cache the result for subsequent tests.
    compiled_components.insert(pkg_dir.to_string(), cdylib_file.clone());
    Ok(cdylib_file)
}

/// Execute the `uniffi-bindgen test` command.
///
/// The default behaviour, suitable for most consumers, is to shell out to the `uniffi-bindgen`
/// command found on the system.
///
/// If the "builtin-bindgen" feature is enabled then this will instead take a direct dependency
/// on the `uniffi_bindgen` crate and execute its methods in-process. This is useful for folks
/// who are working on uniffi itself and want to test out their changes to the bindings generator.
#[cfg(not(feature = "builtin-bindgen"))]
fn run_uniffi_bindgen_test(out_dir: &str, idl_file: &str, test_file: &str) -> Result<()> {
    let status = Command::new("uniffi-bindgen")
        .args(&["test", out_dir, idl_file, test_file])
        .status()?;
    if !status.success() {
        bail!("Error while running tests: {}",);
    }
    Ok(())
}

#[cfg(feature = "builtin-bindgen")]
fn run_uniffi_bindgen_test(out_dir: &str, idl_file: &str, test_file: &str) -> Result<()> {
    uniffi_bindgen::run_tests(out_dir, idl_file, vec![test_file])
}
