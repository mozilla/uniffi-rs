/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use clap::Parser;
use std::env;
use uniffi_benchmarks::Args;
use uniffi_bindgen::bindings::{kotlin, python, swift, RunScriptMode};

fn main() {
    let args = Args::parse();
    let script_args: Vec<String> = std::iter::once(String::from("--"))
        .chain(env::args())
        .collect();

    if args.should_run_python() {
        python::run_script(
            std::env!("CARGO_TARGET_TMPDIR"),
            "uniffi-fixture-benchmarks",
            "benches/bindings/run_benchmarks.py",
            script_args.clone(),
            RunScriptMode::PerformanceTest,
        )
        .unwrap()
    }

    if args.should_run_kotlin() {
        kotlin::run_script(
            std::env!("CARGO_TARGET_TMPDIR"),
            "uniffi-fixture-benchmarks",
            "benches/bindings/run_benchmarks.kts",
            script_args.clone(),
            RunScriptMode::PerformanceTest,
        )
        .unwrap()
    }

    if args.should_run_swift() {
        swift::run_script(
            std::env!("CARGO_TARGET_TMPDIR"),
            "uniffi-fixture-benchmarks",
            "benches/bindings/run_benchmarks.swift",
            script_args.clone(),
            RunScriptMode::PerformanceTest,
        )
        .unwrap()
    }
}
