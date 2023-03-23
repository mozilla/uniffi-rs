/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::time::Duration;

mod cli;
pub use cli::Args;

pub struct TestData {
    foo: String,
    bar: String,
}

pub enum TestCase {
    Function,
    VoidReturn,
    NoArgsVoidReturn,
}

pub trait TestCallbackInterface {
    fn method(&self, a: i32, b: i32, data: TestData) -> String;
    fn method_with_void_return(&self, a: i32, b: i32, data: TestData);
    fn method_with_no_args_and_void_return(&self);
    fn run_test(&self, test_case: TestCase, count: u64) -> u64;
}

pub fn test_function(_a: i32, _b: i32, data: TestData) -> String {
    data.bar
}
pub fn test_void_return(_a: i32, _b: i32, _data: TestData) {}
pub fn test_no_args_void_return() {}

pub fn run_benchmarks(language: String, cb: Box<dyn TestCallbackInterface>) {
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

include!(concat!(env!("OUT_DIR"), "/benchmarks.uniffi.rs"));
