/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use uniffi_benchmarks::Args;
use uniffi_bindgen::bindings::{
    kotlin_test::run_script as kotlin_run_script, python_test::run_script as python_run_script,
    swift_test::run_script as swift_run_script, RunScriptOptions,
};

fn main() -> Result<()> {
    let args = Args::parse();
    let script_args: Vec<String> = std::iter::once(String::from("--"))
        .chain(env::args())
        .collect();

    let measurement_tracker = uniffi_benchmarks::CriterionMeasurementTracker::new()?;

    if let Some(delete) = &args.delete_save {
        measurement_tracker.delete_save(delete)?;
        return Ok(());
    }

    if args.has_compare() && !args.has_save_name() {
        // User is asking to compare, but not save any new args.  We can just print the table
        // without running any benchmarks
        measurement_tracker.compare(&args.compare, args.compare_last, None)?;
        return Ok(());
    }

    let options = RunScriptOptions {
        show_compiler_messages: args.compiler_messages,
    };

    if args.should_run_foreign_language("python") {
        python_run_script(
            std::env!("CARGO_TARGET_TMPDIR"),
            "uniffi-fixture-benchmarks",
            "benches/bindings/run_benchmarks.py",
            script_args.clone(),
            &options,
        )
        .context("Error runing python benchmark script")?;
    }

    if args.should_run_foreign_language("kotlin") {
        kotlin_run_script(
            std::env!("CARGO_TARGET_TMPDIR"),
            "uniffi-fixture-benchmarks",
            "benches/bindings/run_benchmarks.kts",
            script_args.clone(),
            &options,
        )
        .context("Error running kotlin benchmark script")?;
    }

    if args.should_run_foreign_language("swift") {
        swift_run_script(
            std::env!("CARGO_TARGET_TMPDIR"),
            "uniffi-fixture-benchmarks",
            "benches/bindings/run_benchmarks.swift",
            script_args,
            &options,
        )
        .context("Error running Swift benchmark script")?;
    }

    if args.has_compare() || args.has_save_name() {
        let save_name = args
            .has_save_name()
            .then(|| args.calculate_save_name())
            .transpose()?;
        measurement_tracker.compare(&args.compare, args.compare_last, save_name.as_deref())?;
    }
    Ok(())
}
