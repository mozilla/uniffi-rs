/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

mod callback_interface;

use callback_interface::TestCallbackInterface;

#[derive(uniffi::Record)]
pub struct One {
    inner: i32,
}

#[uniffi::export]
pub fn one_inner_by_ref(one: &One) -> i32 {
    one.inner
}

#[derive(uniffi::Record)]
pub struct Two {
    a: String,
    #[uniffi(default = None)]
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

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct RecordWithBytes {
    some_bytes: Vec<u8>,
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

    fn is_other_heavy(&self, other: &Self) -> MaybeBool {
        other.is_heavy()
    }

    fn get_trait(&self, inc: Option<Arc<dyn Trait>>) -> Arc<dyn Trait> {
        inc.unwrap_or_else(|| Arc::new(TraitImpl {}))
    }

    fn take_error(&self, e: BasicError) -> u32 {
        assert!(matches!(e, BasicError::InvalidInput));
        42
    }
}

#[uniffi::export]
fn get_trait_name_by_ref(t: &dyn Trait) -> String {
    t.name()
}

#[uniffi::export]
fn make_one(inner: i32) -> One {
    One { inner }
}

#[uniffi::export]
fn take_two(two: Two) -> String {
    two.a
}

#[uniffi::export]
fn take_record_with_bytes(rwb: RecordWithBytes) -> Vec<u8> {
    rwb.some_bytes
}

#[uniffi::export]
fn test_callback_interface(cb: Box<dyn TestCallbackInterface>) {
    cb.do_nothing();
    assert_eq!(cb.add(1, 1), 2);
    assert_eq!(cb.optional(Some(1)), 1);
    assert_eq!(cb.optional(None), 0);
    assert_eq!(
        cb.with_bytes(RecordWithBytes {
            some_bytes: vec![9, 8, 7],
        }),
        vec![9, 8, 7]
    );
    assert_eq!(Ok(10), cb.try_parse_int("10".to_string()));
    assert_eq!(
        Err(BasicError::InvalidInput),
        cb.try_parse_int("ten".to_string())
    );
    assert!(matches!(
        cb.try_parse_int("force-unexpected-error".to_string()),
        Err(BasicError::UnexpectedError { .. }),
    ));
    assert_eq!(42, cb.callback_handler(Object::new()));
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

#[uniffi::export]
fn make_record_with_bytes() -> RecordWithBytes {
    RecordWithBytes {
        some_bytes: vec![0, 1, 2, 3, 4],
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

#[derive(thiserror::Error, uniffi::Error, Debug, PartialEq, Eq)]
#[uniffi(handle_unknown_callback_error)]
pub enum BasicError {
    #[error("InvalidInput")]
    InvalidInput,
    #[error("OsError")]
    OsError,
    #[error("UnexpectedError")]
    UnexpectedError { reason: String },
}

impl From<uniffi::UnexpectedUniFFICallbackError> for BasicError {
    fn from(e: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::UnexpectedError { reason: e.reason }
    }
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

uniffi::include_scaffolding!("proc-macro");
