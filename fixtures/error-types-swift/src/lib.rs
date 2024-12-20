/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

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

// A conversion into our ErrorInterface from anyhow::Error.
// We can't use this implicitly yet, but it still helps.
impl From<anyhow::Error> for ErrorInterface {
    fn from(e: anyhow::Error) -> Self {
        Self { e }
    }
}

// Test an interface as the error type
fn oops() -> Result<(), Arc<ErrorInterface>> {
    // must do explicit conversion to convert anyhow::Error into ErrorInterface
    Err(Arc::new(
        anyhow::Error::msg("oops")
            .context("because uniffi told me so")
            .into(),
    ))
}

// Like `oops`, but let UniFFI handle wrapping the interface with an arc
fn oops_nowrap() -> Result<(), ErrorInterface> {
    // must do explicit conversion to convert anyhow::Error into ErrorInterface
    Err(anyhow::Error::msg("oops")
        .context("because uniffi told me so")
        .into())
}

#[uniffi::export]
fn toops() -> Result<(), Arc<dyn ErrorTrait>> {
    Err(Arc::new(ErrorTraitImpl {
        m: "trait-oops".to_string(),
    }))
}

#[uniffi::export]
async fn aoops() -> Result<(), Arc<ErrorInterface>> {
    Err(Arc::new(anyhow::Error::msg("async-oops").into()))
}

fn get_error(message: String) -> std::sync::Arc<ErrorInterface> {
    Arc::new(anyhow::Error::msg(message).into())
}

#[uniffi::export]
pub trait ErrorTrait: Send + Sync + std::fmt::Debug + std::error::Error {
    fn msg(&self) -> String;
}

#[derive(Debug, thiserror::Error)]
#[error("{m:?}")]
struct ErrorTraitImpl {
    m: String,
}

impl ErrorTrait for ErrorTraitImpl {
    fn msg(&self) -> String {
        self.m.clone()
    }
}

fn throw_rich(e: String) -> Result<(), RichError> {
    Err(RichError { e })
}

// Exists to test trailing "Error" mapping in bindings
#[derive(Debug, thiserror::Error)]
#[error("RichError: {e:?}")]
pub struct RichError {
    e: String,
}

impl RichError {}

pub struct TestInterface {}

impl TestInterface {
    fn new() -> Self {
        TestInterface {}
    }

    fn fallible_new() -> Result<Self, Arc<ErrorInterface>> {
        Err(Arc::new(anyhow::Error::msg("fallible_new").into()))
    }

    fn oops(&self) -> Result<(), Arc<ErrorInterface>> {
        Err(Arc::new(
            anyhow::Error::msg("oops")
                .context("because the interface told me so")
                .into(),
        ))
    }
}

#[uniffi::export]
impl TestInterface {
    // can't define this in UDL due to #1915
    async fn aoops(&self) -> Result<(), Arc<ErrorInterface>> {
        Err(Arc::new(anyhow::Error::msg("async-oops").into()))
    }
}

// A procmacro as an error
#[derive(Debug, uniffi::Object, thiserror::Error)]
#[uniffi::export(Debug, Display)]
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
        write!(f, "ProcErrorInterface({})", self.e)
    }
}

#[uniffi::export]
fn throw_proc_error(e: String) -> Result<(), Arc<ProcErrorInterface>> {
    Err(Arc::new(ProcErrorInterface { e }))
}

#[uniffi::export]
fn return_proc_error(e: String) -> Arc<ProcErrorInterface> {
    Arc::new(ProcErrorInterface { e })
}

#[derive(thiserror::Error, Debug)]
#[error("NonUniffiTypeValue: {v}")]
pub struct NonUniffiType {
    v: String,
}

// Note: It's important for this test that this error
// *not* be used directly as the `Err` for any functions etc.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum Inner {
    #[error("{0}")]
    CaseA(String),
}

// Note: It's important for this test that this error
// *not* be used directly as the `Err` for any functions etc.
#[derive(thiserror::Error, uniffi::Error, Debug)]
#[uniffi(flat_error)]
pub enum FlatInner {
    #[error("{0}")]
    CaseA(String),
    #[error("{0}")]
    CaseB(NonUniffiType),
}

// Enums have good coverage elsewhere, but simple coverage here is good.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum Error {
    #[error("Oops")]
    Oops,
    #[error("Value: {value}")]
    Value { value: String },
    #[error("IntValue: {value}")]
    IntValue { value: u16 },
    #[error(transparent)]
    FlatInnerError {
        #[from]
        error: FlatInner,
    },
    #[error(transparent)]
    InnerError { error: Inner },
}

#[uniffi::export]
fn oops_enum(i: u16) -> Result<(), Error> {
    if i == 0 {
        Err(Error::Oops)
    } else if i == 1 {
        Err(Error::Value {
            value: "value".to_string(),
        })
    } else if i == 2 {
        Err(Error::IntValue { value: i })
    } else if i == 3 {
        Err(Error::FlatInnerError {
            error: FlatInner::CaseA("inner".to_string()),
        })
    } else if i == 4 {
        Err(Error::FlatInnerError {
            error: FlatInner::CaseB(NonUniffiType {
                v: "value".to_string(),
            }),
        })
    } else if i == 5 {
        Err(Error::InnerError {
            error: Inner::CaseA("inner".to_string()),
        })
    } else {
        panic!("unknown variant {i}")
    }
}

// tuple enum as an error.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum TupleError {
    #[error("Oops")]
    Oops(String),
    #[error("Value {0}")]
    Value(u16),
}

#[uniffi::export]
fn oops_tuple(i: u16) -> Result<(), TupleError> {
    if i == 0 {
        Err(TupleError::Oops("oops".to_string()))
    } else if i == 1 {
        Err(TupleError::Value(i))
    } else {
        panic!("unknown variant {i}")
    }
}

#[uniffi::export(default(t = None))]
fn get_tuple(t: Option<TupleError>) -> TupleError {
    t.unwrap_or_else(|| TupleError::Oops("oops".to_string()))
}

uniffi::include_scaffolding!("error_types");
