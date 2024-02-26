/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, sync::Arc};

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
    // Test the absence of `with_foreign` by inputting reference arguments, which is
    // incompatible with callback interfaces
    #[allow(clippy::ptr_arg)]
    fn concat_strings(&self, a: &str, b: &str) -> String;
}

struct TraitImpl {}

impl Trait for TraitImpl {
    fn concat_strings(&self, a: &str, b: &str) -> String {
        format!("{a}{b}")
    }
}

#[uniffi::export(with_foreign)]
pub trait TraitWithForeign: Send + Sync {
    fn name(&self) -> String;
}

struct RustTraitImpl {}

impl TraitWithForeign for RustTraitImpl {
    fn name(&self) -> String {
        "RustTraitImpl".to_string()
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
    fn named_ctor(arg: u32) -> Self {
        _ = arg;
        // This constructor returns Self directly.  UniFFI ensures that it's wrapped in an Arc
        // before sending it across the FFI.
        Self
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

    fn get_trait_with_foreign(
        &self,
        inc: Option<Arc<dyn TraitWithForeign>>,
    ) -> Arc<dyn TraitWithForeign> {
        inc.unwrap_or_else(|| Arc::new(RustTraitImpl {}))
    }

    fn take_error(&self, e: BasicError) -> u32 {
        assert!(matches!(e, BasicError::InvalidInput));
        42
    }
}

#[uniffi::export]
fn concat_strings_by_ref(t: &dyn Trait, a: &str, b: &str) -> String {
    t.concat_strings(a, b)
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
fn make_hashmap(k: i8, v: u64) -> HashMap<i8, u64> {
    HashMap::from([(k, v)])
}

// XXX - fails to call this from python - https://github.com/mozilla/uniffi-rs/issues/1774
#[uniffi::export]
fn return_hashmap(h: HashMap<i8, u64>) -> HashMap<i8, u64> {
    h
}

#[uniffi::export]
fn take_record_with_bytes(rwb: RecordWithBytes) -> Vec<u8> {
    rwb.some_bytes
}

#[uniffi::export]
fn call_callback_interface(cb: Box<dyn TestCallbackInterface>) {
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

    assert_eq!(6, cb.get_other_callback_interface().multiply(2, 3));
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

#[derive(uniffi::Enum)]
pub enum MixedEnum {
    None,
    String(String),
    Int(i64),
    Both(String, i64),
    All { s: String, i: i64 },
}

#[uniffi::export]
fn get_mixed_enum(v: Option<MixedEnum>) -> MixedEnum {
    v.unwrap_or(MixedEnum::Int(1))
}

#[repr(u8)]
#[derive(uniffi::Enum)]
pub enum ReprU8 {
    One = 1,
    Three = 0x3,
}

#[uniffi::export]
fn enum_identity(value: MaybeBool) -> MaybeBool {
    value
}

#[derive(thiserror::Error, uniffi::Error, Debug, PartialEq, Eq)]
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

// defined in UDL.
fn get_one(one: Option<One>) -> One {
    one.unwrap_or(One { inner: 0 })
}

fn get_bool(b: Option<MaybeBool>) -> MaybeBool {
    b.unwrap_or(MaybeBool::Uncertain)
}

fn get_object(o: Option<Arc<Object>>) -> Arc<Object> {
    o.unwrap_or_else(Object::new)
}

fn get_trait(o: Option<Arc<dyn Trait>>) -> Arc<dyn Trait> {
    o.unwrap_or_else(|| Arc::new(TraitImpl {}))
}

fn get_trait_with_foreign(o: Option<Arc<dyn TraitWithForeign>>) -> Arc<dyn TraitWithForeign> {
    o.unwrap_or_else(|| Arc::new(RustTraitImpl {}))
}

#[derive(Default)]
struct Externals {
    one: Option<One>,
    bool: Option<MaybeBool>,
}

fn get_externals(e: Option<Externals>) -> Externals {
    e.unwrap_or_default()
}

#[uniffi::export]
pub fn join(parts: &[String], sep: &str) -> String {
    parts.join(sep)
}

// Custom names
#[derive(uniffi::Object)]
pub struct Renamed;

// `renamed_new` becomes the default constructor because it's named `new`
#[uniffi::export]
impl Renamed {
    #[uniffi::constructor(name = "new")]
    fn renamed_new() -> Arc<Self> {
        Arc::new(Self)
    }

    #[uniffi::method(name = "func")]
    fn renamed_func(&self) -> bool {
        true
    }
}

#[uniffi::export(name = "rename_test")]
fn renamed_rename_test() -> bool {
    true
}

uniffi::include_scaffolding!("proc-macro");
