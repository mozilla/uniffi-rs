/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{process::Command, time::Duration};

use anyhow::{bail, Result};
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

    /// Save benchmark data, using the JJ change ID of the parent commit
    #[clap(long, conflicts_with("save"))]
    pub save_with_jj_parent: bool,

    /// Save benchmark data, using the JJ change ID of the current commit
    #[clap(long, conflicts_with("save"), conflicts_with("save_with_jj_parent"))]
    pub save_with_jj_commit: bool,

    /// Delete benchmark data previously recorded with save
    #[clap(long)]
    pub delete_save: Option<String>,

    /// Create a table comparing previously saved benchmark data
    ///
    /// Inputs a list of names previously passed to `--save`.
    /// Each name will be a column in the table.
    ///
    /// If `--save` is also present, then new measurements will be added as a column in the table.
    /// If not, then this will skip new measurements and only print out a table.
    #[clap(short, long, use_value_delimiter = true)]
    pub compare: Vec<String>,

    /// Create a table comparing previously saved benchmark data
    ///
    /// This works like `--compare`, instead of inputting save names, this compares against the last
    /// N save points.
    #[clap(long)]
    pub compare_last: Option<usize>,

    /// Run for a fixed number of seconds and skip the analysis.
    ///
    /// Use this for hooking up a profile to the benchmark code.
    #[clap(long)]
    pub profile_time: Option<u64>,

    // Args for running the metrics, these are handled in `lib.rs`
    /// Only run benchmarks whose names contain FILTER
    /// Multiple arguments are ANDed together
    #[clap()]
    pub filter: Vec<String>,

    // It would be great to also support the baseline arguments, but there doesn't seem to be any
    // way to manually set those.

    // Ignore the `--bench` arg, which Cargo passes to us
    #[clap(long, hide = true)]
    bench: bool,
}

impl Args {
    /// Should we run the tests for a foreign language?
    pub fn should_run_foreign_language(&self, language: &str) -> bool {
        TestCase::all_names_for_language(language)
            .any(|name| self.test_case_name_matches_filter(&name))
    }

    pub fn test_case_name_matches_filter(&self, name: &str) -> bool {
        self.filter.iter().all(|filter| name.contains(filter))
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
        if let Some(profile_time) = self.profile_time {
            c = c.profile_time(Some(Duration::from_secs(profile_time)));
        }
        c
    }

    pub fn skip_measurements(&self) -> bool {
        !self.has_save_name() && !self.has_compare()
    }

    pub fn has_compare(&self) -> bool {
        !self.compare.is_empty() || self.compare_last.is_some()
    }

    pub fn has_save_name(&self) -> bool {
        self.save.is_some() || self.save_with_jj_parent || self.save_with_jj_commit
    }

    pub fn calculate_save_name(&self) -> Result<String> {
        if let Some(name) = &self.save {
            Ok(name.clone())
        } else if self.save_with_jj_commit {
            self.determine_save_name_from_jj_commit()
        } else if self.save_with_jj_parent {
            self.determine_save_name_from_jj_parent()
        } else {
            bail!("Save name not found")
        }
    }

    pub fn determine_save_name_from_jj_commit(&self) -> Result<String> {
        let output = Command::new("jj")
            .args(["log", "-r", "@", "-G", "-T", "change_id.short()"])
            .output()?;
        if !output.status.success() {
            bail!(
                "jj log failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(str::from_utf8(&output.stdout)?.trim().to_string())
    }

    pub fn determine_save_name_from_jj_parent(&self) -> Result<String> {
        let output = Command::new("jj")
            .args([
                "log",
                "-r",
                "ancestors(@,2)",
                "-G",
                "-T",
                "concat(change_id.short(),'\n')",
            ])
            .output()?;
        if !output.status.success() {
            bail!(
                "jj log failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let lines = str::from_utf8(&output.stdout)?
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        match lines.as_slice() {
            [] | [_] => bail!("No JJ parent commit"),
            [_, parent_change_id] => Ok(parent_change_id.to_string()),
            [_, _, ..] => bail!("Multiple JJ parent commits"),
        }
    }
}
