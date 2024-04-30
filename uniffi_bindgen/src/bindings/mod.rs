/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generate foreign language bindings for a uniffi component.
//!
//! This module contains all the code for generating foreign language bindings,
//! along with some helpers for executing foreign language scripts or tests.

use anyhow::{bail, Result};

use std::fmt;

mod kotlin;
pub use kotlin::{
    run_script as kotlin_run_script, run_test as kotlin_run_test, KotlinBindingGenerator,
};
mod python;
pub use python::{
    run_script as python_run_script, run_test as python_run_test, PythonBindingGenerator,
};
mod ruby;
pub use ruby::{run_test as ruby_run_test, RubyBindingGenerator};
mod swift;
pub use swift::{
    run_script as swift_run_script, run_test as swift_run_test, SwiftBindingGenerator,
};

/// Enumeration of all foreign language targets currently supported by this crate.
///
/// The functions in this module will delegate to a language-specific backend based
/// on the provided `TargetLanguage`. For convenience of calling code we also provide
/// a few `TryFrom` implementations to help guess the correct target language from
/// e.g. a file extension of command-line argument.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum TargetLanguage {
    Kotlin,
    Swift,
    Python,
    Ruby,
}

impl fmt::Display for TargetLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kotlin => write!(f, "kotlin"),
            Self::Swift => write!(f, "swift"),
            Self::Python => write!(f, "python"),
            Self::Ruby => write!(f, "ruby"),
        }
    }
}

/// Mode for the `run_script` function defined for each language
#[derive(Clone, Debug)]
pub struct RunScriptOptions {
    pub show_compiler_messages: bool,
}

impl Default for RunScriptOptions {
    fn default() -> Self {
        Self {
            show_compiler_messages: true,
        }
    }
}

impl TryFrom<&str> for TargetLanguage {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "kotlin" | "kt" | "kts" => TargetLanguage::Kotlin,
            "swift" => TargetLanguage::Swift,
            "python" | "py" => TargetLanguage::Python,
            "ruby" | "rb" => TargetLanguage::Ruby,
            _ => bail!("Unknown or unsupported target language: \"{value}\""),
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
