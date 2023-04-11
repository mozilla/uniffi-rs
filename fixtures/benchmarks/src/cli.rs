/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use clap::Parser;
use criterion::Criterion;

#[derive(Parser, Debug)]
pub struct Args {
    // Args to select which test scripts run.  These are handled in `benchmarks.rs`.
    /// Run Python tests
    #[clap(short, long = "py", display_order = 0)]
    pub python: bool,
    /// Run Kotlin tests
    #[clap(short, long = "kt", display_order = 0)]
    pub kotlin: bool,
    /// Run Swift tests
    #[clap(short, long, display_order = 0)]
    pub swift: bool,
    /// Run Dart tests
    #[clap(short, long, display_order = 0)]
    pub dart: bool,

    /// Dump compiler output to the console.  Good for debugging new benchmarks.
    #[clap(long, display_order = 1)]
    pub compiler_messages: bool,

    // Args for running the metrics, these are handled in `lib.rs`
    /// Skip benchmarks whose names do not contain FILTER
    #[clap()]
    pub filter: Option<String>,

    // It would be great to also support the baseline arguments, but there doesn't seem to be any
    // way to manually set those.

    // Ignore the `--bench` arg, which Cargo passes to us
    #[clap(long, hide = true)]
    bench: bool,
}

impl Args {
    /// Should we run the Python tests?
    pub fn should_run_python(&self) -> bool {
        self.python || self.no_languages_selected()
    }

    /// Should we run the Kotlin tests?
    pub fn should_run_kotlin(&self) -> bool {
        self.kotlin || self.no_languages_selected()
    }

    /// Should we run the Swift tests?
    pub fn should_run_swift(&self) -> bool {
        self.swift || self.no_languages_selected()
    }

    pub fn no_languages_selected(&self) -> bool {
        !(self.python || self.kotlin || self.swift)
    }

    /// Parse arguments for run_benchmarks()
    ///
    /// This is slightly tricky, because run_benchmarks() is called from the foreign bindings side.
    /// This means that `sys::env::args()` will contain all the arguments needed to run the foreign
    /// bindings script, and we don't want to parse all of that.
    pub fn parse_for_run_benchmarks() -> Self {
        Self::parse_from(
            std::env::args()
                .into_iter()
                // This method finds the first "--" arg, which `benchmarks.rs` inserts to mark the start of
                // our arguments.
                .skip_while(|a| a != "--")
                // Skip over any "--" args.  This is mainly for the "--" arg that we use to separate
                // our args.  However, it's also needed to workaround some kotlinc behavior.  We need
                // to pass it the "--" arg to get it to start trying to parse our arguments, but
                // kotlinc still passes "--" back to us.
                .skip_while(|a| a == "--")
                .collect::<Vec<String>>(),
        )
    }

    /// Build a Criterion instance from the arguments
    pub fn build_criterion(&self) -> Criterion {
        let mut c = Criterion::default();
        c = match &self.filter {
            Some(f) => c.with_filter(f),
            None => c,
        };
        c
    }
}
