/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

fn anyhow_bail(message: String) -> anyhow::Result<()> {
    anyhow::bail!("{message}");
}

fn anyhow_with_context(message: String) -> anyhow::Result<()> {
    Err(anyhow::Error::msg(message).context("because uniffi told me so"))
}

fn get_error(message: String) -> std::sync::Arc<ErrorInterface> {
    std::sync::Arc::new(ErrorInterface {
        e: anyhow::Error::msg(message),
    })
}

#[derive(Debug, thiserror::Error)]
#[error("{e:?}")]
pub struct ErrorInterface {
    e: anyhow::Error,
}

impl ErrorInterface {
    fn chain(&self) -> Vec<String> {
        self.e.chain().map(ToString::to_string).collect()
    }
    fn link(&self, ndx: u64) -> Option<String> {
        self.e.chain().nth(ndx as usize).map(ToString::to_string)
    }
}

impl From<anyhow::Error> for ErrorInterface {
    fn from(e: anyhow::Error) -> Self {
        Self { e }
    }
}

fn throw_rich(e: String) -> Result<(), RichError> {
    Err(RichError { e })
}

fn get_rich_error(e: String) -> std::sync::Arc<RichError> {
    std::sync::Arc::new(RichError { e })
}

#[derive(Debug, thiserror::Error)]
#[error("RichError: {e:?}")]
pub struct RichError {
    e: String,
}

impl RichError {}

struct TestInterface {}

impl TestInterface {
    fn new() -> Self {
        TestInterface {}
    }

    fn fallible_new() -> anyhow::Result<Self> {
        Err(anyhow::Error::msg("fallible_new"))
    }

    fn anyhow_bail(&self, message: String) -> anyhow::Result<()> {
        anyhow::bail!("TestInterface - {message}");
    }
}

// A procmacro as an error
#[derive(Debug, uniffi::Error, thiserror::Error)]
pub struct ProcErrorInterface {
    e: String,
}

#[uniffi::export]
impl ProcErrorInterface {
    fn message(&self) -> String {
        self.e.clone()
    }
}

impl std::fmt::Display for ProcErrorInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProcErrorInterface {}", self.e)
    }
}

// XXX - sadly no `From` support, making this not useful for structs etc returning
// `anyhow::Error` - need ability to tell the procmaco what types to use in the ffi functions?
#[uniffi::export]
fn throw_proc_error(e: String) -> Result<(), Arc<ProcErrorInterface>> {
    Err(Arc::new(ProcErrorInterface { e }))
}

#[uniffi::export]
fn return_proc_error(e: String) -> Arc<ProcErrorInterface> {
    Arc::new(ProcErrorInterface { e })
}

uniffi::include_scaffolding!("error_types");
