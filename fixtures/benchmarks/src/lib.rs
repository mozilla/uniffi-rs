/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, sync::Arc, time::Duration};

mod cli;
mod compare;
pub use cli::Args;
pub use compare::CriterionMeasurementTracker;

/// Benchmark test cases
///
/// Each variant represent an FFI call that passes/returns a particular kind of data.
#[derive(uniffi::Enum, Clone, Copy, Debug)]
pub enum TestCase {
    CallOnly,
    Primitives,
    Strings,
    LargeStrings,
    Records,
    LargeRecords,
    Enums,
    Hashmaps,
    Interfaces,
    TraitInterfaces,
    Optionals,
    Bytes,
    VecSmall,
    VecPrimitives,
    VecRecords,
    Methods,
    NestedData,
    Errors,
}

impl TestCase {
    fn iter_all() -> impl Iterator<Item = Self> {
        [
            Self::CallOnly,
            Self::Primitives,
            Self::Strings,
            Self::LargeStrings,
            Self::Records,
            Self::LargeRecords,
            Self::Enums,
            Self::Hashmaps,
            Self::Interfaces,
            Self::TraitInterfaces,
            Self::NestedData,
            Self::Optionals,
            Self::Bytes,
            Self::VecSmall,
            Self::VecPrimitives,
            Self::VecRecords,
            Self::Methods,
            Self::Errors,
        ]
        .into_iter()
    }

    fn name(&self) -> &'static str {
        match self {
            Self::CallOnly => "call-only",
            Self::Primitives => "primitives",
            Self::Strings => "strings",
            Self::LargeStrings => "large-strings",
            Self::Records => "records",
            Self::LargeRecords => "large-records",
            Self::Enums => "enums",
            Self::Hashmaps => "hash-maps",
            Self::Interfaces => "interfaces",
            Self::TraitInterfaces => "trait-interfaces",
            Self::NestedData => "nested-data",
            Self::Optionals => "optionals",
            Self::Bytes => "bytes",
            Self::VecSmall => "vecs-small",
            Self::VecPrimitives => "vec-primitives",
            Self::VecRecords => "vec-records",
            Self::Methods => "methods",
            Self::Errors => "errors",
        }
    }

    fn rust_to_foreign_name(&self, language: &str) -> String {
        format!("{language}-rust-{}", self.name())
    }

    fn foreign_to_rust_name(&self, language: &str) -> String {
        format!("rust-{language}-{}", self.name())
    }

    fn all_names_for_language(language: &str) -> impl Iterator<Item = String> {
        let language = language.to_string();
        Self::iter_all().flat_map(move |test_case| {
            [
                test_case.rust_to_foreign_name(&language),
                test_case.foreign_to_rust_name(&language),
            ]
        })
    }

    pub fn skip_foreign_side(&self) -> bool {
        // The methods tests only makes sense on Rust.
        // The equivalent on the foreign side is the `TestCase::CallOnly`.
        matches!(self, Self::Methods)
    }

    fn callback_test_case_fn(&self, cb: Arc<dyn TestCallbackInterface>) -> Box<dyn Fn()> {
        match self {
            TestCase::CallOnly => Box::new(move || cb.call_only()),
            TestCase::Primitives => Box::new(move || {
                cb.primitives(0, -1);
            }),
            TestCase::Strings => Box::new(move || {
                cb.strings("a".to_string(), "b".to_string());
            }),
            TestCase::LargeStrings => {
                let large_string1 = "a".repeat(2048);
                let large_string2 = "b".repeat(1500);
                Box::new(move || {
                    cb.large_strings(large_string1.clone(), large_string2.clone());
                })
            }
            TestCase::Records => Box::new(move || {
                cb.records(
                    TestRecord {
                        a: -1,
                        b: 1,
                        c: 1.5,
                    },
                    TestRecord {
                        a: -2,
                        b: 2,
                        c: 4.5,
                    },
                );
            }),
            TestCase::LargeRecords => Box::new(move || {
                cb.large_records(
                    TestLargeRecord {
                        a: 1,
                        b: 2,
                        c: 3,
                        d: 4,
                        e: 1.0,
                        f: 2.0,
                        g: true,
                    },
                    TestLargeRecord {
                        a: 1,
                        b: 2,
                        c: 3,
                        d: 4,
                        e: 1.0,
                        f: 2.0,
                        g: true,
                    },
                );
            }),
            TestCase::Enums => Box::new(move || {
                cb.enums(TestEnum::One { a: -1, b: 0 }, TestEnum::Two { c: 1.5 });
            }),
            TestCase::VecSmall => Box::new(move || {
                cb.vec_small(vec![0, 1], vec![2, 4, 6]);
            }),
            TestCase::Hashmaps => Box::new(move || {
                cb.hash_maps(HashMap::from([(0, 1), (1, 2)]), HashMap::from([(2, 4)]));
            }),
            TestCase::Interfaces => Box::new(move || {
                cb.interfaces(Arc::new(TestInterface), Arc::new(TestInterface));
            }),
            TestCase::TraitInterfaces => Box::new(move || {
                cb.trait_interfaces(
                    Arc::new(TestTraitInterfaceImpl),
                    Arc::new(TestTraitInterfaceImpl),
                );
            }),
            TestCase::Optionals => Box::new(move || {
                cb.optionals(Some(10), None, Some("testing-123".into()));
            }),
            TestCase::Bytes => {
                let vec: Vec<u8> = (0..=255).collect();
                Box::new(move || {
                    cb.bytes(vec.clone());
                })
            }
            TestCase::VecPrimitives => {
                let vec: Vec<u32> = (0..1024).collect();
                Box::new(move || {
                    cb.vec_primitives(vec.clone());
                })
            }
            TestCase::VecRecords => {
                let vec: Vec<TestRecord> = (0..1024)
                    .map(|i| TestRecord {
                        a: i,
                        b: i as u64 * 2,
                        c: i as f64 / 2.0,
                    })
                    .collect();
                Box::new(move || {
                    cb.vec_records(vec.clone());
                })
            }
            TestCase::Methods => {
                panic!("Attempt to run TestCase::Method from the foreign side");
            }
            TestCase::NestedData => Box::new(move || {
                cb.nested_data(
                    NestedData {
                        a: vec![TestRecord {
                            a: -1,
                            b: 1,
                            c: 1.5,
                        }],
                        b: vec![
                            vec!["one".to_string(), "two".to_string()],
                            vec!["three".to_string()],
                        ],
                        c: HashMap::from([
                            ("one".to_string(), TestEnum::One { a: -1, b: 1 }),
                            ("two".to_string(), TestEnum::Two { c: 0.5 }),
                        ]),
                    },
                    NestedData {
                        a: vec![TestRecord {
                            a: -2,
                            b: 2,
                            c: 2.5,
                        }],
                        b: vec![vec!["four".to_string()], vec!["five".to_string()]],
                        c: HashMap::from([("two".to_string(), TestEnum::Two { c: -0.5 })]),
                    },
                );
            }),
            TestCase::Errors => Box::new(move || {
                let _ = cb.errors();
            }),
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct TestRecord {
    a: i32,
    b: u64,
    c: f64,
}

#[derive(uniffi::Record)]
pub struct TestLargeRecord {
    a: i8,
    b: i16,
    c: i32,
    d: i64,
    e: f32,
    f: f64,
    g: bool,
}

#[derive(uniffi::Enum)]
pub enum TestEnum {
    One { a: i32, b: u64 },
    Two { c: f64 },
}

#[derive(uniffi::Object, Default)]
pub struct TestInterface;

#[uniffi::export]
impl TestInterface {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }
}

#[uniffi::export]
pub trait TestTraitInterface: Send + Sync {}

struct TestTraitInterfaceImpl;

impl TestTraitInterface for TestTraitInterfaceImpl {}

#[uniffi::export]
pub fn make_test_trait_interface() -> Arc<dyn TestTraitInterface> {
    Arc::new(TestTraitInterfaceImpl)
}

#[derive(uniffi::Record)]
pub struct NestedData {
    a: Vec<TestRecord>,
    b: Vec<Vec<String>>,
    c: HashMap<String, TestEnum>,
}

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum TestError {
    #[error("One")]
    One,
    #[error("Two")]
    Two,
}

// Test functions
//
// One for each `TestCase` variant.  Test cases are implemented by having the foreign side call
// these repeatedly.
//
// These ignore the inputs and return a fixed return value.  This should be okay for benchmarking
// purposes, assuming that compilers can't optimize calls when they cross the FFI.

#[uniffi::export]
pub fn test_case_call_only() {}

#[uniffi::export]
pub fn test_case_primitives(a: u8, b: i32) -> f64 {
    (a as f64) + (b as f64)
}

#[uniffi::export]
pub fn test_case_strings(a: String, b: String) -> String {
    a + &b
}

#[uniffi::export]
pub fn test_case_large_strings(a: String, b: String) -> String {
    a + &b
}

#[uniffi::export]
pub fn test_case_records(a: TestRecord, b: TestRecord) -> TestRecord {
    TestRecord {
        a: a.a + b.a,
        b: a.b + b.b,
        c: a.c + b.c,
    }
}

#[uniffi::export]
pub fn test_case_large_records(a: TestLargeRecord, b: TestLargeRecord) -> TestLargeRecord {
    TestLargeRecord {
        a: a.a + b.a,
        b: a.b + b.b,
        c: a.c + b.c,
        d: a.d + b.d,
        e: a.e + b.e,
        f: a.f + b.f,
        g: a.g || b.g,
    }
}

#[uniffi::export]
pub fn test_case_enums(a: TestEnum, b: TestEnum) -> TestEnum {
    let a_sum = match a {
        TestEnum::One { a, b } => a as f64 + b as f64,
        TestEnum::Two { c } => c,
    };
    let b_sum = match b {
        TestEnum::One { a, b } => a as f64 + b as f64,
        TestEnum::Two { c } => c,
    };
    TestEnum::Two { c: a_sum + b_sum }
}

#[uniffi::export]
pub fn test_case_hashmaps(a: HashMap<u32, u32>, b: HashMap<u32, u32>) -> HashMap<u32, u32> {
    a.into_iter().chain(b).collect()
}

#[uniffi::export]
pub fn test_case_interfaces(a: Arc<TestInterface>, b: Arc<TestInterface>) -> Arc<TestInterface> {
    // Perform some silliness to make sure Rust needs to access both `a` and `b`
    if Arc::strong_count(&a) > 100 {
        a
    } else {
        b
    }
}

#[uniffi::export]
pub fn test_case_trait_interfaces(
    a: Arc<dyn TestTraitInterface>,
    b: Arc<dyn TestTraitInterface>,
) -> Arc<dyn TestTraitInterface> {
    // Perform some silliness to make sure Rust needs to access both `a` and `b`
    if Arc::strong_count(&a) > 100 {
        a
    } else {
        b
    }
}

#[uniffi::export]
pub fn test_case_optionals(a: Option<u32>, b: Option<bool>, c: Option<String>) -> u32 {
    let mut sum = 0;
    if let Some(value) = a {
        sum += value;
    }
    if let Some(true) = b {
        sum *= 2;
    }
    if let Some(string) = c {
        sum += string.len() as u32;
    }
    sum
}

#[uniffi::export]
pub fn test_case_bytes(v: Vec<u8>) -> Vec<u8> {
    v
}

#[uniffi::export]
pub fn test_case_vec_small(a: Vec<u32>, b: Vec<u32>) -> Vec<u32> {
    a.into_iter().chain(b).collect()
}

#[uniffi::export]
pub fn test_case_vec_primitives(v: Vec<u32>) -> Vec<u32> {
    v
}

#[uniffi::export]
pub fn test_case_vec_records(v: Vec<TestRecord>) -> Vec<TestRecord> {
    v
}

#[uniffi::export]
pub fn test_case_nested_data(a: NestedData, b: NestedData) -> NestedData {
    NestedData {
        a: a.a.into_iter().chain(b.a).collect(),
        b: a.b.into_iter().chain(b.b).collect(),
        c: a.c.into_iter().chain(b.c).collect(),
    }
}

#[uniffi::export]
pub fn test_case_errors() -> Result<u32, TestError> {
    Err(TestError::Two)
}

#[uniffi::export]
impl TestInterface {
    pub fn noop_method(&self) {}
}

/// Benchmarks callback interface
///
/// This contains a method for each [TestCase] variant.  To test callback methods, the Rust code
/// will repeatedly invoke these methods on a foreign implementation.
#[uniffi::export(with_foreign)]
pub trait TestCallbackInterface: Send + Sync {
    fn call_only(&self);
    fn primitives(&self, a: u8, b: i32) -> f64;
    fn strings(&self, a: String, b: String) -> String;
    fn large_strings(&self, a: String, b: String) -> String;
    fn records(&self, a: TestRecord, b: TestRecord) -> TestRecord;
    fn large_records(&self, a: TestLargeRecord, b: TestLargeRecord) -> TestLargeRecord;
    fn enums(&self, a: TestEnum, b: TestEnum) -> TestEnum;
    fn hash_maps(&self, a: HashMap<u32, u32>, b: HashMap<u32, u32>) -> HashMap<u32, u32>;
    fn interfaces(&self, a: Arc<TestInterface>, b: Arc<TestInterface>) -> Arc<TestInterface>;
    fn trait_interfaces(
        &self,
        a: Arc<dyn TestTraitInterface>,
        b: Arc<dyn TestTraitInterface>,
    ) -> Arc<dyn TestTraitInterface>;
    fn optionals(&self, a: Option<u32>, b: Option<bool>, c: Option<String>) -> u32;
    fn bytes(&self, v: Vec<u8>) -> Vec<u8>;
    fn vec_small(&self, a: Vec<u32>, b: Vec<u32>) -> Vec<u32>;
    fn vec_primitives(&self, v: Vec<u32>) -> Vec<u32>;
    fn vec_records(&self, v: Vec<TestRecord>) -> Vec<TestRecord>;
    fn nested_data(&self, a: NestedData, b: NestedData) -> NestedData;
    fn errors(&self) -> Result<u32, TestError>;

    /// Run a [TestCase] test N times by calling the corresponding Rust function and return the
    /// elapsed time in nanoseconds.  The Rust benchmarks calls this to run the test case, calling
    /// these functions from Rust will clearly not give the correct results.
    fn run_test(&self, test_case: TestCase, count: u64) -> u64;
}

/// Call a callback method N times
///
/// This is used by the Android app to run callback benchmarks.  Trying to invoke the callback
/// method from the foreign side will clearly not give the correct results.
#[uniffi::export]
pub fn run_callback_test(cb: Arc<dyn TestCallbackInterface>, test_case: TestCase, n: u64) {
    // Note: `std::hint::black_box` is not needed, since we're making an FFI call.  There's no way
    // Rust can optimize that out.

    let case = test_case.callback_test_case_fn(cb);
    for _ in 0..n {
        case()
    }
}

/// Run all benchmarks and print the results to stdout
#[uniffi::export]
pub fn run_benchmarks(language: String, cb: Arc<dyn TestCallbackInterface>) {
    let args = Args::parse_for_run_benchmarks();

    let measurement_tracker =
        CriterionMeasurementTracker::new().expect("Error creating CriterionMeasurementTracker");

    let mut c = args.build_criterion();

    for test_case in TestCase::iter_all() {
        let rust_to_foreign_name = test_case.rust_to_foreign_name(&language);
        let foreign_to_rust_name = test_case.foreign_to_rust_name(&language);

        if args.test_case_name_matches_filter(&rust_to_foreign_name) {
            c.bench_function(&rust_to_foreign_name, |b| {
                b.iter_custom(|count| Duration::from_nanos(cb.run_test(test_case, count)))
            });
        }

        if !test_case.skip_foreign_side()
            && args.test_case_name_matches_filter(&foreign_to_rust_name)
        {
            let callback_fn = test_case.callback_test_case_fn(cb.clone());
            c.bench_function(&foreign_to_rust_name, move |b| {
                b.iter(&callback_fn);
            });
        }
    }

    c.final_summary();

    if args.has_save_name() {
        let name = args.calculate_save_name().expect("Error saving times");
        measurement_tracker.save(&name).expect("Error saving times");
    }
}

uniffi::setup_scaffolding!("benchmarks");
