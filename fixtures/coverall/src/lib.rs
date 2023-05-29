/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::SystemTime;

use once_cell::sync::Lazy;

mod traits;
pub use traits::{get_traits, TestTrait};

static NUM_ALIVE: Lazy<RwLock<u64>> = Lazy::new(|| RwLock::new(0));

#[derive(Debug, thiserror::Error)]
pub enum CoverallError {
    #[error("The coverall has too many holes")]
    TooManyHoles,
}

#[derive(Debug, thiserror::Error)]
pub enum CoverallFlatError {
    #[error("Too many variants: {num}")]
    TooManyVariants { num: i16 },
}

fn throw_flat_error() -> Result<(), CoverallFlatError> {
    Err(CoverallFlatError::TooManyVariants { num: 99 })
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)] // "flat" isn't really the correct terminology here.
pub enum CoverallMacroError {
    #[error("The coverall has too many macros")]
    TooManyMacros,
}

#[uniffi::export]
fn throw_macro_error() -> Result<(), CoverallMacroError> {
    Err(CoverallMacroError::TooManyMacros)
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum CoverallFlatMacroError {
    #[error("Too many variants: {num}")]
    TooManyVariants { num: i16 },
}

#[uniffi::export]
fn throw_flat_macro_error() -> Result<(), CoverallFlatMacroError> {
    Err(CoverallFlatMacroError::TooManyVariants { num: 88 })
}

pub enum CoverallRichErrorNoVariantData {
    TooManyPlainVariants,
}

fn throw_rich_error_no_variant_data() -> Result<(), CoverallRichErrorNoVariantData> {
    Err(CoverallRichErrorNoVariantData::TooManyPlainVariants)
}

/// This error doesn't appear in the interface, instead
/// we rely on an `Into<CoverallError>` impl to surface it to consumers.
#[derive(Debug, thiserror::Error)]
pub enum InternalCoverallError {
    #[error("The coverall has an excess of holes")]
    ExcessiveHoles,
}

impl From<InternalCoverallError> for CoverallError {
    fn from(err: InternalCoverallError) -> CoverallError {
        match err {
            InternalCoverallError::ExcessiveHoles => CoverallError::TooManyHoles,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ComplexError {
    #[error("OsError: {code} ({extended_code})")]
    OsError { code: i16, extended_code: i16 },
    #[error("PermissionDenied: {reason}")]
    PermissionDenied { reason: String },
    #[error("Unknown error")]
    UnknownError,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ComplexMacroError {
    #[error("OsError: {code} ({extended_code})")]
    OsError { code: i16, extended_code: i16 },
    #[error("PermissionDenied: {reason}")]
    PermissionDenied { reason: String },
    #[error("Unknown error")]
    UnknownError,
}

#[uniffi::export]
fn throw_complex_macro_error() -> Result<(), ComplexMacroError> {
    Err(ComplexMacroError::OsError {
        code: 1,
        extended_code: 2,
    })
}

#[derive(Clone, Debug, Default)]
pub struct SimpleDict {
    text: String,
    maybe_text: Option<String>,
    some_bytes: Vec<u8>,
    maybe_some_bytes: Option<Vec<u8>>,
    a_bool: bool,
    maybe_a_bool: Option<bool>,
    unsigned8: u8,
    maybe_unsigned8: Option<u8>,
    unsigned16: u16,
    maybe_unsigned16: Option<u16>,
    unsigned64: u64,
    maybe_unsigned64: Option<u64>,
    signed8: i8,
    maybe_signed8: Option<i8>,
    signed64: i64,
    maybe_signed64: Option<i64>,
    float32: f32,
    maybe_float32: Option<f32>,
    float64: f64,
    maybe_float64: Option<f64>,
    coveralls: Option<Arc<Coveralls>>,
    test_trait: Option<Arc<dyn TestTrait>>,
}

#[derive(Debug, Clone)]
pub struct DictWithDefaults {
    name: String,
    category: Option<String>,
    integer: u64,
}

#[derive(Debug, Clone)]
pub enum MaybeSimpleDict {
    Yeah { d: SimpleDict },
    Nah,
}

fn get_maybe_simple_dict(index: i8) -> MaybeSimpleDict {
    match index {
        0 => MaybeSimpleDict::Yeah {
            d: SimpleDict::default(),
        },
        1 => MaybeSimpleDict::Nah,
        _ => unreachable!("invalid index: {index}"),
    }
}

// UDL can not describe this as a "flat" enum, but we'll keep it here to help demonstrate that!
#[derive(Debug, Clone)]
pub enum SimpleFlatEnum {
    First { val: String },
    Second { num: u16 },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum SimpleFlatMacroEnum {
    First { val: String },
    Second { num: u16 },
}

#[uniffi::export]
fn get_simple_flat_macro_enum(index: i8) -> SimpleFlatMacroEnum {
    match index {
        0 => SimpleFlatMacroEnum::First {
            val: "the first".to_string(),
        },
        1 => SimpleFlatMacroEnum::Second { num: 2 },
        _ => unreachable!("invalid index: {index}"),
    }
}

fn create_some_dict() -> SimpleDict {
    SimpleDict {
        text: "text".to_string(),
        maybe_text: Some("maybe_text".to_string()),
        some_bytes: b"some_bytes".to_vec(),
        maybe_some_bytes: Some(b"maybe_some_bytes".to_vec()),
        a_bool: true,
        maybe_a_bool: Some(false),
        unsigned8: 1,
        maybe_unsigned8: Some(2),
        unsigned16: 3,
        maybe_unsigned16: Some(4),
        unsigned64: u64::MAX,
        maybe_unsigned64: Some(u64::MIN),
        signed8: 8,
        maybe_signed8: Some(0),
        signed64: i64::MAX,
        maybe_signed64: Some(0),
        float32: 1.2345,
        maybe_float32: Some(22.0 / 7.0),
        float64: 0.0,
        maybe_float64: Some(1.0),
        coveralls: Some(Arc::new(Coveralls::new("some_dict".to_string()))),
        test_trait: Some(Arc::new(traits::Trait2 {})),
    }
}

fn create_none_dict() -> SimpleDict {
    SimpleDict {
        text: "text".to_string(),
        some_bytes: b"some_bytes".to_vec(),
        a_bool: true,
        unsigned8: 1,
        unsigned16: 3,
        unsigned64: u64::MAX,
        signed8: 8,
        signed64: i64::MAX,
        float32: 1.2345,
        ..Default::default()
    }
}

fn get_num_alive() -> u64 {
    *NUM_ALIVE.read().unwrap()
}

type Result<T, E = CoverallError> = std::result::Result<T, E>;
type ComplexResult<T, E = ComplexError> = std::result::Result<T, E>;

fn println(text: String) -> Result<()> {
    println!("coveralls println: {text}");
    Ok(())
}

#[derive(Debug)]
pub struct Coveralls {
    name: String,
    // A reference to another Coveralls. Currently will be only a reference
    // to `self`, so will create a circular reference.
    other: Mutex<Option<Arc<Self>>>,
    // Repairs we've made to this coverall.
    repairs: Mutex<Vec<Repair>>,
}

impl Coveralls {
    fn new(name: String) -> Self {
        *NUM_ALIVE.write().unwrap() += 1;
        Self {
            name,
            other: Mutex::new(None),
            repairs: Mutex::new(Vec::new()),
        }
    }

    fn fallible_new(name: String, should_fail: bool) -> Result<Self> {
        if should_fail {
            Err(CoverallError::TooManyHoles)
        } else {
            Ok(Self::new(name))
        }
    }

    fn fallible_panic(&self, message: String) -> Result<()> {
        panic!("{message}");
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn panicking_new(message: String) -> Self {
        panic!("{message}");
    }

    fn maybe_throw(&self, should_throw: bool) -> Result<bool> {
        if should_throw {
            Err(CoverallError::TooManyHoles)
        } else {
            Ok(true)
        }
    }

    fn maybe_throw_into(&self, should_throw: bool) -> Result<bool, InternalCoverallError> {
        if should_throw {
            Err(InternalCoverallError::ExcessiveHoles)
        } else {
            Ok(true)
        }
    }

    fn maybe_throw_complex(&self, input: i8) -> ComplexResult<bool> {
        match input {
            0 => Ok(true),
            1 => Err(ComplexError::OsError {
                code: 10,
                extended_code: 20,
            }),
            2 => Err(ComplexError::PermissionDenied {
                reason: "Forbidden".to_owned(),
            }),
            3 => Err(ComplexError::UnknownError),
            _ => panic!("Invalid input"),
        }
    }

    fn panic(&self, message: String) {
        panic!("{message}");
    }

    fn strong_count(self: Arc<Self>) -> u64 {
        Arc::strong_count(&self) as u64
    }

    fn take_other(&self, other: Option<Arc<Self>>) {
        *self.other.lock().unwrap() = other.map(|arc| Arc::clone(&arc))
    }

    fn get_other(&self) -> Option<Arc<Self>> {
        (*self.other.lock().unwrap()).as_ref().map(Arc::clone)
    }

    fn take_other_fallible(self: Arc<Self>) -> Result<()> {
        Err(CoverallError::TooManyHoles)
    }

    fn take_other_panic(self: Arc<Self>, message: String) {
        panic!("{message}");
    }

    fn clone_me(&self) -> Arc<Self> {
        let other = self.other.lock().unwrap();
        let new_other = Mutex::new(other.clone());
        *NUM_ALIVE.write().unwrap() += 1;
        Arc::new(Self {
            name: self.name.clone(),
            other: new_other,
            repairs: Mutex::new(Vec::new()),
        })
    }

    fn get_status(&self, status: String) -> String {
        format!("status: {status}")
    }

    fn get_dict(&self, key: String, value: u64) -> HashMap<String, u64> {
        let mut map = HashMap::new();
        map.insert(key, value);
        map
    }

    fn get_dict2(&self, key: String, value: u64) -> HashMap<String, u64> {
        let mut map = HashMap::new();
        map.insert(key, value);
        map
    }

    fn get_dict3(&self, key: u32, value: u64) -> HashMap<u32, u64> {
        let mut map = HashMap::new();
        map.insert(key, value);
        map
    }

    fn add_patch(&self, patch: Arc<Patch>) {
        let repair = Repair {
            when: SystemTime::now(),
            patch,
        };
        let mut repairs = self.repairs.lock().unwrap();
        repairs.push(repair);
    }

    fn add_repair(&self, repair: Repair) {
        let mut repairs = self.repairs.lock().unwrap();
        repairs.push(repair);
    }

    fn get_repairs(&self) -> Vec<Repair> {
        let repairs = self.repairs.lock().unwrap();
        repairs.clone()
    }

    fn reverse(&self, mut value: Vec<u8>) -> Vec<u8> {
        value.reverse();
        value
    }
}

impl Drop for Coveralls {
    fn drop(&mut self) {
        *NUM_ALIVE.write().unwrap() -= 1;
    }
}

#[derive(Debug, Clone)]
pub struct Repair {
    when: SystemTime,
    patch: Arc<Patch>,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Blue,
    Green,
}

#[derive(Debug, Clone)]
struct Patch {
    color: Color,
}

impl Patch {
    fn new(color: Color) -> Self {
        Self { color }
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

// This is a small implementation of a counter that allows waiting on one thread,
// and counting on another thread. We use it to test that the UniFFI generated scaffolding
// doesn't introduce unexpected locking behaviour between threads.
struct ThreadsafeCounter {
    is_busy: AtomicBool,
    count: AtomicI32,
}

impl ThreadsafeCounter {
    fn new() -> Self {
        Self {
            is_busy: AtomicBool::new(false),
            count: AtomicI32::new(0),
        }
    }

    fn busy_wait(&self, ms: i32) {
        self.is_busy.store(true, Ordering::SeqCst);
        // Pretend to do some work in a blocking fashion.
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
        self.is_busy.store(false, Ordering::SeqCst);
    }

    fn increment_if_busy(&self) -> i32 {
        if self.is_busy.load(Ordering::SeqCst) {
            self.count.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            self.count.load(Ordering::SeqCst)
        }
    }
}

uniffi::include_scaffolding!("coverall");
