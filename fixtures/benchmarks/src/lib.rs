/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::time::Duration;

mod cli;
pub use cli::Args;

#[derive(uniffi::Record)]
pub struct TestData {
    foo: String,
    bar: String,
}

#[derive(uniffi::Enum)]
pub enum TestCase {
    Function,
    VoidReturn,
    NoArgsVoidReturn,
}

/// Test callback methods.
///
/// These are intended to test the overhead of callback interface calls
/// including: popping arguments from the stack, unpacking RustBuffers,
/// pushing return values back to the stack, etc.
#[uniffi::export(with_foreign)]
pub trait TestCallbackInterface: Send + Sync {
    fn method(&self, a: i32, b: i32, data: TestData) -> String;
    fn method_with_void_return(&self, a: i32, b: i32, data: TestData);
    fn method_with_no_args_and_void_return(&self);
    /// Run a performance test N times and return the elapsed time in nanoseconds
    fn run_test(&self, test_case: TestCase, count: u64) -> u64;
}

/// Test functions
///
/// These are intended to test the overhead of Rust function calls including:
/// popping arguments from the stack, unpacking RustBuffers, pushing return
/// values back to the stack, etc.
#[uniffi::export]
pub fn test_function(_a: i32, _b: i32, data: TestData) -> String {
    data.bar
}

#[uniffi::export]
pub fn test_void_return(_a: i32, _b: i32, _data: TestData) {}
#[uniffi::export]
pub fn test_no_args_void_return() {}

/// Run all benchmarks and print the results to stdout
#[uniffi::export]
pub fn run_benchmarks(language: String, cb: Arc<dyn TestCallbackInterface>) {
    let args = Args::parse_for_run_benchmarks();
    let mut c = args.build_criterion();

    c.benchmark_group("calls")
        // FFI Function call benchmarks
        //
        // Note: these are more a proof-of-concept than real benchmarks.  Before using these to
        // drive changes, make sure to take some time and double check that they're testing the
        // correct things.
        .bench_function(format!("{language}-functions-basic"), |b| {
            b.iter_custom(|count| Duration::from_nanos(cb.run_test(TestCase::Function, count)))
        })
        .bench_function(format!("{language}-functions-void-return"), |b| {
            b.iter_custom(|count| Duration::from_nanos(cb.run_test(TestCase::VoidReturn, count)))
        })
        .bench_function(format!("{language}-functions-no-args-void-return"), |b| {
            b.iter_custom(|count| {
                Duration::from_nanos(cb.run_test(TestCase::NoArgsVoidReturn, count))
            })
        });

    c.benchmark_group("callbacks")
        // These benchmarks are extra noisy, take extra time to measure them and set a higher noise
        // threshold
        .measurement_time(Duration::from_secs(10))
        .noise_threshold(0.05)
        .bench_function(format!("{language}-callbacks-basic"), |b| {
            b.iter(|| {
                cb.method(
                    10,
                    100,
                    TestData {
                        foo: String::from("SomeStringData"),
                        bar: String::from("SomeMoreStringData"),
                    },
                )
            })
        })
        .bench_function(format!("{language}-callbacks-void-return"), |b| {
            b.iter(|| {
                cb.method_with_void_return(
                    10,
                    100,
                    TestData {
                        foo: String::from("SomeStringData"),
                        bar: String::from("SomeMoreStringData"),
                    },
                )
            })
        })
        .bench_function(format!("{language}-callbacks-no-args-void-return"), |b| {
            b.iter(|| cb.method_with_no_args_and_void_return())
        });

    c.final_summary();
}

uniffi::setup_scaffolding!("benchmarks");
