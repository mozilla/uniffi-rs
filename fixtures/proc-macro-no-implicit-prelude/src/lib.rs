/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#![no_implicit_prelude]

// Let's not have the macros care about derive being shadowed in the macro namespace for now..
use ::std::prelude::rust_2021::derive;
// Required for now because `static-assertions` (used internally) macros assume it to be in scope.
use ::std::marker::Sized;

mod callback_interface;

use callback_interface::TestCallbackInterface;

#[derive(::uniffi::Record)]
pub struct One {
    inner: i32,
}

#[::uniffi::export]
pub fn one_inner_by_ref(one: &One) -> i32 {
    one.inner
}

#[derive(::uniffi::Record)]
pub struct Two {
    a: ::std::string::String,
}

#[derive(::uniffi::Record)]
pub struct NestedRecord {
    // This used to result in an error in bindings generation
    user_type_in_builtin_generic: ::std::option::Option<Two>,
}

#[derive(::uniffi::Record)]
pub struct Three {
    obj: ::std::sync::Arc<Object>,
}

#[derive(::uniffi::Record, Debug, PartialEq)]
pub struct RecordWithBytes {
    some_bytes: ::std::vec::Vec<u8>,
}

// An object that's not used anywhere (ie, in records, function signatures, etc)
// should not break things.
#[derive(::uniffi::Object)]
pub struct Unused;

#[::uniffi::export]
impl Unused {
    #[::uniffi::constructor]
    fn new() -> ::std::sync::Arc<Self> {
        ::std::sync::Arc::new(Self)
    }
}

#[::uniffi::export]
pub trait Trait: ::std::marker::Send + ::std::marker::Sync {
    // Test the absence of `with_foreign` by inputting reference arguments, which is
    // incompatible with callback interfaces
    #[allow(clippy::ptr_arg)]
    fn concat_strings(&self, a: &str, b: &str) -> ::std::string::String;
}

struct TraitImpl {}

impl Trait for TraitImpl {
    fn concat_strings(&self, a: &str, b: &str) -> ::std::string::String {
        ::std::format!("{a}{b}")
    }
}

#[::uniffi::export(with_foreign)]
pub trait TraitWithForeign: ::std::marker::Send + ::std::marker::Sync {
    fn name(&self) -> ::std::string::String;
}

struct RustTraitImpl {}

impl TraitWithForeign for RustTraitImpl {
    fn name(&self) -> ::std::string::String {
        use ::std::string::ToString;
        "RustTraitImpl".to_string()
    }
}

#[derive(::uniffi::Object)]
pub struct Object;

#[cfg_attr(feature = "myfeature", ::uniffi::export)]
impl Object {
    #[cfg_attr(feature = "myfeature", ::uniffi::constructor)]
    fn new() -> ::std::sync::Arc<Self> {
        ::std::sync::Arc::new(Self)
    }

    #[::uniffi::constructor]
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

    fn get_trait(
        &self,
        inc: ::std::option::Option<::std::sync::Arc<dyn Trait>>,
    ) -> ::std::sync::Arc<dyn Trait> {
        inc.unwrap_or_else(|| ::std::sync::Arc::new(TraitImpl {}))
    }

    fn get_trait_with_foreign(
        &self,
        inc: ::std::option::Option<::std::sync::Arc<dyn TraitWithForeign>>,
    ) -> ::std::sync::Arc<dyn TraitWithForeign> {
        inc.unwrap_or_else(|| ::std::sync::Arc::new(RustTraitImpl {}))
    }

    fn take_error(&self, e: BasicError) -> u32 {
        ::std::assert!(::std::matches!(e, BasicError::InvalidInput));
        42
    }
}

#[::uniffi::export]
fn concat_strings_by_ref(t: &dyn Trait, a: &str, b: &str) -> ::std::string::String {
    t.concat_strings(a, b)
}

#[::uniffi::export]
fn make_one(inner: i32) -> One {
    One { inner }
}

#[::uniffi::export]
fn take_two(two: Two) -> ::std::string::String {
    two.a
}

#[::uniffi::export]
fn make_hashmap(k: i8, v: u64) -> ::std::collections::HashMap<i8, u64> {
    ::std::convert::From::from([(k, v)])
}

#[::uniffi::export]
fn return_hashmap(h: ::std::collections::HashMap<i8, u64>) -> ::std::collections::HashMap<i8, u64> {
    h
}

#[::uniffi::export]
fn take_record_with_bytes(rwb: RecordWithBytes) -> ::std::vec::Vec<u8> {
    rwb.some_bytes
}

#[::uniffi::export]
fn call_callback_interface(cb: ::std::boxed::Box<dyn TestCallbackInterface>) {
    use ::std::{assert_eq, matches, option::Option::*, result::Result::*, string::ToString, vec};

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

// Type that's defined in the UDL and not wrapped with #[::uniffi::export]
pub struct Zero {
    inner: ::std::string::String,
}

#[::uniffi::export]
fn make_zero() -> Zero {
    use ::std::borrow::ToOwned;
    Zero {
        inner: "ZERO".to_owned(),
    }
}

#[::uniffi::export]
fn make_record_with_bytes() -> RecordWithBytes {
    RecordWithBytes {
        some_bytes: ::std::vec![0, 1, 2, 3, 4],
    }
}

#[derive(::uniffi::Enum)]
pub enum MaybeBool {
    True,
    False,
    Uncertain,
}

#[derive(::uniffi::Enum)]
pub enum MixedEnum {
    None,
    String(::std::string::String),
    Int(i64),
    Both(::std::string::String, i64),
    All { s: ::std::string::String, i: i64 },
}

#[::uniffi::export]
fn get_mixed_enum(v: ::std::option::Option<MixedEnum>) -> MixedEnum {
    v.unwrap_or(MixedEnum::Int(1))
}

#[repr(u8)]
#[derive(::uniffi::Enum)]
pub enum ReprU8 {
    One = 1,
    Three = 0x3,
}

#[::uniffi::export]
fn enum_identity(value: MaybeBool) -> MaybeBool {
    value
}

#[derive(::uniffi::Error, Debug, PartialEq, Eq)]
pub enum BasicError {
    InvalidInput,
    OsError,
    UnexpectedError { reason: ::std::string::String },
}

impl ::std::fmt::Display for BasicError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "InvalidInput",
            Self::OsError => "OsError",
            Self::UnexpectedError { .. } => "UnexpectedError",
        })
    }
}

impl ::std::error::Error for BasicError {}

impl ::std::convert::From<::uniffi::UnexpectedUniFFICallbackError> for BasicError {
    fn from(e: ::uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::UnexpectedError { reason: e.reason }
    }
}

#[::uniffi::export]
fn always_fails() -> ::std::result::Result<(), BasicError> {
    ::std::result::Result::Err(BasicError::OsError)
}

#[derive(Debug, ::uniffi::Error)]
#[uniffi(flat_error)]
#[non_exhaustive]
pub enum FlatError {
    InvalidInput,

    // Inner types that aren't FFI-convertible, as well as unnamed fields,
    // are allowed for flat errors
    OsError(::std::io::Error),
}

impl ::std::fmt::Display for FlatError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("Invalid input"),
            Self::OsError(e) => ::std::write!(f, "OS error: {e}"),
        }
    }
}

#[::uniffi::export]
impl Object {
    fn do_stuff(&self, times: u32) -> ::std::result::Result<(), FlatError> {
        match times {
            0 => ::std::result::Result::Err(FlatError::InvalidInput),
            _ => {
                // do stuff
                ::std::result::Result::Ok(())
            }
        }
    }
}

// defined in UDL.
fn get_one(one: ::std::option::Option<One>) -> One {
    one.unwrap_or(One { inner: 0 })
}

fn get_bool(b: ::std::option::Option<MaybeBool>) -> MaybeBool {
    b.unwrap_or(MaybeBool::Uncertain)
}

fn get_object(o: ::std::option::Option<::std::sync::Arc<Object>>) -> ::std::sync::Arc<Object> {
    o.unwrap_or_else(Object::new)
}

fn get_trait(o: ::std::option::Option<::std::sync::Arc<dyn Trait>>) -> ::std::sync::Arc<dyn Trait> {
    o.unwrap_or_else(|| ::std::sync::Arc::new(TraitImpl {}))
}

fn get_trait_with_foreign(
    o: ::std::option::Option<::std::sync::Arc<dyn TraitWithForeign>>,
) -> ::std::sync::Arc<dyn TraitWithForeign> {
    o.unwrap_or_else(|| ::std::sync::Arc::new(RustTraitImpl {}))
}

#[derive(Default)]
struct Externals {
    one: ::std::option::Option<One>,
    bool: ::std::option::Option<MaybeBool>,
}

fn get_externals(e: ::std::option::Option<Externals>) -> Externals {
    e.unwrap_or_default()
}

#[::uniffi::export]
pub fn join(parts: &[::std::string::String], sep: &str) -> ::std::string::String {
    parts.join(sep)
}

// Custom names
#[derive(::uniffi::Object)]
pub struct Renamed;

// `renamed_new` becomes the default constructor because it's named `new`
#[::uniffi::export]
impl Renamed {
    #[::uniffi::constructor(name = "new")]
    fn renamed_new() -> ::std::sync::Arc<Self> {
        ::std::sync::Arc::new(Self)
    }

    #[::uniffi::method(name = "func")]
    fn renamed_func(&self) -> bool {
        true
    }
}

#[::uniffi::export(name = "rename_test")]
fn renamed_rename_test() -> bool {
    true
}

/// Test defaults on Records
#[derive(::uniffi::Record)]
pub struct RecordWithDefaults {
    no_default_string: ::std::string::String,
    #[uniffi(default = true)]
    boolean: bool,
    #[uniffi(default = 42)]
    integer: i32,
    #[uniffi(default = 4.2)]
    float_var: f64,
    #[uniffi(default=[])]
    vec: ::std::vec::Vec<bool>,
    #[uniffi(default=None)]
    opt_vec: ::std::option::Option<::std::vec::Vec<bool>>,
    #[uniffi(default = Some(42))]
    opt_integer: ::std::option::Option<i32>,
}

/// Test defaults on top-level functions
#[::uniffi::export(default(num = 21))]
fn double_with_default(num: i32) -> i32 {
    num + num
}

/// Test defaults on constructors / methods
#[derive(::uniffi::Object)]
pub struct ObjectWithDefaults {
    num: i32,
}

#[::uniffi::export]
impl ObjectWithDefaults {
    #[::uniffi::constructor(default(num = 30))]
    fn new(num: i32) -> Self {
        Self { num }
    }

    #[::uniffi::method(default(other = 12))]
    fn add_to_num(&self, other: i32) -> i32 {
        self.num + other
    }
}

::uniffi::include_scaffolding!("proc-macro");
