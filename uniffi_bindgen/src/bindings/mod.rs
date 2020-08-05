/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generate foreign language bindings for a uniffi component.
//!
//! This module contains all the code for generating foreign language bindings,
//! along with some helpers for executing foreign language scripts or tests.

use anyhow::{bail, Result};
use std::convert::{TryFrom, TryInto};
use std::path::Path;

use crate::interface::ComponentInterface;

pub mod kotlin;
pub mod python;
pub mod swift;

/// Enumeration of all foreign language targets currently supported by this crate.
///
/// The functions in this module will delegate to a language-specific backend based
/// on the provided `TargetLanguage`. For convenience of calling code we also provide
/// a few `TryFrom` implementations to help guess the correct target language from
/// e.g. a file extension of command-line argument.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum TargetLanguage {
    Kotlin,
    Swift,
    Python,
}

impl TryFrom<&str> for TargetLanguage {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "kotlin" | "kt" | "kts" => TargetLanguage::Kotlin,
            "swift" => TargetLanguage::Swift,
            "python" | "py" => TargetLanguage::Python,
            _ => bail!("Unknown or unsupported target language: \"{}\"", value),
        })
    }
}

impl TryFrom<&std::ffi::OsStr> for TargetLanguage {
    type Error = anyhow::Error;
    fn try_from(value: &std::ffi::OsStr) -> Result<Self> {
        match value.to_str() {
            None => bail!("Unreadable target language"),
            Some(s) => s.try_into(),
        }
    }
}

impl TryFrom<String> for TargetLanguage {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        TryFrom::try_from(value.as_str())
    }
}

/// Generate foreign language bindings from a compiled `uniffi` library.
pub fn write_bindings<P>(
    ci: &ComponentInterface,
    out_dir: P,
    language: TargetLanguage,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let out_dir = out_dir.as_ref();
    match language {
        TargetLanguage::Kotlin => kotlin::write_bindings(&ci, out_dir)?,
        TargetLanguage::Swift => swift::write_bindings(&ci, out_dir)?,
        TargetLanguage::Python => python::write_bindings(&ci, out_dir)?,
    }
    Ok(())
}

/// Compile generated foreign language bindings so they're ready for use.
pub fn compile_bindings<P>(
    ci: &ComponentInterface,
    out_dir: P,
    language: TargetLanguage,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let out_dir = out_dir.as_ref();
    match language {
        TargetLanguage::Kotlin => kotlin::compile_bindings(&ci, out_dir)?,
        TargetLanguage::Swift => swift::compile_bindings(&ci, out_dir)?,
        TargetLanguage::Python => (),
    }
    Ok(())
}

/// Execute the given script via foreign language interpreter/shell.
pub fn run_script<P1, P2>(out_dir: P1, script_file: P2, language: TargetLanguage) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let out_dir = out_dir.as_ref();
    let script_file = script_file.as_ref();
    match language {
        TargetLanguage::Kotlin => kotlin::run_script(out_dir, script_file)?,
        TargetLanguage::Swift => swift::run_script(out_dir, script_file)?,
        TargetLanguage::Python => python::run_script(out_dir, script_file)?,
    }
    Ok(())
}
