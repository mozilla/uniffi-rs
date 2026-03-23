/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::time::Duration;

use clap::Parser;
use criterion::Criterion;

use crate::TestCase;

#[derive(Parser, Debug)]
pub struct Args {
    /// Dump compiler output to the console.  Good for debugging new benchmarks.
    #[clap(long)]
    pub compiler_messages: bool,

    /// Save benchmark data for later comparisons
    #[clap(short, long)]
    pub save: Option<String>,

    /// Create a table comparing previously saved benchmark data
    ///
    /// Inputs a list of names previously passed to `--save`.
    /// Each name will be a column in the table.
    ///
    /// If `--save` is also present, then new measurements will be added as a column in the table.
    /// If not, then this will skip new measurements and only print out a table.
    #[clap(short, long, use_value_delimiter = true)]
    pub compare: Vec<String>,

    /// Run for a fixed number of seconds and skip the analysis.
    ///
    /// Use this for hooking up a profile to the benchmark code.
    #[clap(long)]
    pub profile_time: Option<u64>,

    // Args for running the metrics, these are handled in `lib.rs`
    /// Only run benchmarks whose names contain FILTER
    #[clap()]
    pub filter: Option<String>,

    // It would be great to also support the baseline arguments, but there doesn't seem to be any
    // way to manually set those.

    // Ignore the `--bench` arg, which Cargo passes to us
    #[clap(long, hide = true)]
    bench: bool,
}

impl Args {
    /// Should we run the tests for a foreign language?
    pub fn should_run_foreign_language(&self, language: &str) -> bool {
        match &self.filter {
            None => true,
            Some(filter) => {
                TestCase::all_names_for_language(language).any(|name| name.contains(filter))
            }
        }
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

    /// Build a Criterion instance from the arguments
    pub fn build_criterion(&self) -> Criterion {
        let mut c = Criterion::default();
        c = match &self.filter {
            None => c,
            Some(filter) => c.with_filter(filter),
        };
        if let Some(profile_time) = self.profile_time {
            c = c.profile_time(Some(Duration::from_secs(profile_time)));
        }
        c
    }

    pub fn skip_measurements(&self) -> bool {
        self.save.is_none() && !self.compare.is_empty()
    }

    pub fn compare_table_columns(&self) -> Option<Vec<&str>> {
        (!self.compare.is_empty()).then(|| {
            self.compare
                .iter()
                .map(String::as_str)
                .chain(self.save.as_deref())
                .collect()
        })
    }
}
