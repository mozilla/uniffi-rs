/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, sync::Arc, time::Duration};

mod cli;
pub use cli::Args;

/// Benchmark test cases
///
/// Each variant represent an FFI call that passes/returns a particular kind of data.
#[derive(uniffi::Enum, Clone, Copy)]
pub enum TestCase {
    CallOnly,
    Primitives,
    Strings,
    LargeStrings,
    Records,
    Enums,
    Vecs,
    Hashmaps,
    Interfaces,
    TraitInterfaces,
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
            Self::Enums,
            Self::Vecs,
            Self::Hashmaps,
            Self::Interfaces,
            Self::TraitInterfaces,
            Self::NestedData,
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
            Self::Enums => "enums",
            Self::Vecs => "vecs",
            Self::Hashmaps => "hash-maps",
            Self::Interfaces => "interfaces",
            Self::TraitInterfaces => "trait-interfaces",
            Self::NestedData => "nested-data",
            Self::Errors => "errors",
        }
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
            TestCase::LargeStrings => Box::new(move || {
                cb.large_strings("a".repeat(2048), "b".repeat(1500));
            }),
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
            TestCase::Enums => Box::new(move || {
                cb.enums(TestEnum::One { a: -1, b: 0 }, TestEnum::Two { c: 1.5 });
            }),
            TestCase::Vecs => Box::new(move || {
                cb.vecs(vec![0, 1], vec![2, 4, 6]);
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

#[derive(uniffi::Record)]
pub struct TestRecord {
    a: i32,
    b: u64,
    c: f64,
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
pub fn test_case_vecs(a: Vec<u32>, b: Vec<u32>) -> Vec<u32> {
    a.into_iter().chain(b).collect()
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
    fn enums(&self, a: TestEnum, b: TestEnum) -> TestEnum;
    fn vecs(&self, a: Vec<u32>, b: Vec<u32>) -> Vec<u32>;
    fn hash_maps(&self, a: HashMap<u32, u32>, b: HashMap<u32, u32>) -> HashMap<u32, u32>;
    fn interfaces(&self, a: Arc<TestInterface>, b: Arc<TestInterface>) -> Arc<TestInterface>;
    fn trait_interfaces(
        &self,
        a: Arc<dyn TestTraitInterface>,
        b: Arc<dyn TestTraitInterface>,
    ) -> Arc<dyn TestTraitInterface>;
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
    let mut c = args.build_criterion();

    {
        let mut function_calls = c.benchmark_group("function-calls");
        for test_case in TestCase::iter_all() {
            function_calls.bench_function(format!("{language}-{}", test_case.name()), |b| {
                b.iter_custom(|count| Duration::from_nanos(cb.run_test(test_case, count)))
            });
        }
    }
    {
        let mut callbacks = c.benchmark_group("callbacks");
        for test_case in TestCase::iter_all() {
            let test_case_fn = test_case.callback_test_case_fn(cb.clone());
            callbacks.bench_function(format!("{language}-{}", test_case.name()), move |b| {
                b.iter(&test_case_fn);
            });
        }
    }

    c.final_summary();
}

uniffi::setup_scaffolding!("benchmarks");
