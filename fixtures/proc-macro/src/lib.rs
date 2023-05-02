/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct One {
    inner: i32,
}

#[derive(uniffi::Record)]
pub struct Two {
    a: String,
    b: Option<Vec<bool>>,
}

#[derive(uniffi::Record)]
pub struct NestedRecord {
    // This used to result in an error in bindings generation
    user_type_in_builtin_generic: Option<Two>,
}

#[derive(uniffi::Record)]
pub struct Three {
    obj: Arc<Object>,
}

// An object that's not used anywhere (ie, in records, function signatures, etc)
// should not break things.
#[derive(uniffi::Object)]
pub struct Unused;

#[uniffi::export]
impl Unused {
    #[uniffi::constructor]
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[uniffi::export]
pub trait Trait: Send + Sync {
    fn name(&self) -> String;
}

struct TraitImpl {}

impl Trait for TraitImpl {
    fn name(&self) -> String {
        "TraitImpl".to_string()
    }
}

#[derive(uniffi::Object)]
pub struct Object;

#[uniffi::export]
impl Object {
    #[uniffi::constructor]
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    #[uniffi::constructor]
    fn named_ctor(arg: u32) -> Arc<Self> {
        _ = arg;
        Self::new()
    }

    fn is_heavy(&self) -> MaybeBool {
        MaybeBool::Uncertain
    }

    fn get_trait(&self, inc: Option<Arc<dyn Trait>>) -> Arc<dyn Trait> {
        inc.unwrap_or_else(|| Arc::new(TraitImpl {}))
    }
}

#[uniffi::export]
fn make_one(inner: i32) -> One {
    One { inner }
}

#[uniffi::export]
fn take_two(two: Two) -> String {
    two.a
}

// Type that's defined in the UDL and not wrapped with #[uniffi::export]
pub struct Zero {
    inner: String,
}

#[uniffi::export]
fn make_zero() -> Zero {
    Zero {
        inner: String::from("ZERO"),
    }
}

#[derive(uniffi::Enum)]
pub enum MaybeBool {
    True,
    False,
    Uncertain,
}

#[uniffi::export]
fn enum_identity(value: MaybeBool) -> MaybeBool {
    value
}

#[derive(uniffi::Error)]
pub enum BasicError {
    InvalidInput,
    OsError,
}

#[uniffi::export]
fn always_fails() -> Result<(), BasicError> {
    Err(BasicError::OsError)
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
#[non_exhaustive]
pub enum FlatError {
    #[error("Invalid input")]
    InvalidInput,

    // Inner types that aren't FFI-convertible, as well as unnamed fields,
    // are allowed for flat errors
    #[error("OS error: {0}")]
    OsError(std::io::Error),
}

#[uniffi::export]
impl Object {
    fn do_stuff(&self, times: u32) -> Result<(), FlatError> {
        match times {
            0 => Err(FlatError::InvalidInput),
            _ => {
                // do stuff
                Ok(())
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/proc-macro.uniffi.rs"));
