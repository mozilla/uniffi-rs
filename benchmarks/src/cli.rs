/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use clap::Parser;

use crate::TestCase;

/// Args passed by the user to the main binary
#[derive(Parser, Debug)]
pub struct Args {
    /// Save measurements as a baseline
    #[clap(short, long)]
    pub save: Option<String>,

    /// Compare to one or more previously saved baselines
    #[clap(short, long, value_delimiter=',')]
    pub compare: Vec<String>,

    /// Dump compiler output to the console.  Good for debugging new benchmarks.
    #[clap(long)]
    pub compiler_messages: bool,

    /// Only run benchmarks whose names contain FILTER
    /// Multiple filters will be ANDed together.
    pub filter: Vec<String>,

    /// Delete old benchmarks
    #[clap(long)]
    pub clean: bool,
}

impl Args {
    pub fn matches_test(&self, name: &str) -> bool {
        self.filter.iter().all(|f| name.contains(f))
    }

    /// Should we run the Python tests?
    pub fn should_run_python(&self) -> bool {
        self.matches_language("python")
    }

    /// Should we run the Kotlin tests?
    pub fn should_run_kotlin(&self) -> bool {
        self.matches_language("kotlin")
    }

    /// Should we run the Swift tests?
    pub fn should_run_swift(&self) -> bool {
        self.matches_language("swift")
    }

    fn matches_language(&self, language: &str) -> bool {
        for test_case in TestCase::iter_all() {
            if self.matches_test(&format!("rust-to-{language}/{}", test_case.name())) {
                return true;
            }
            if self.matches_test(&format!("{language}-to-rust/{}", test_case.name())) {
                return true;
            }
        }
        false
    }

    /// Parse arguments for run_benchmarks()
    ///
    /// This is slightly tricky, because run_benchmarks() is called from the foreign bindings side.
    /// This means that `sys::env::args()` will contain all the arguments needed to run the foreign
    /// bindings script, and we don't want to parse all of that.
    pub fn parse_for_run_benchmarks() -> Self {
        Self::parse_from(
            std::env::args()
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
}
