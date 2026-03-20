/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, fs, io::Write};

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use serde::{Deserialize, Serialize};

use crate::ITERATIONS;

#[derive(Deserialize, Serialize)]
pub struct BenchmarkMeasurements {
    pub iterations: u64,
    /// Map test case names to samples.
    ///
    /// The unit is ns / call
    pub samples: HashMap<String, Vec<u64>>,
}

impl BenchmarkMeasurements {
    pub fn load(name: &str) -> Result<Self> {
        let dir = Self::save_dir()?;
        let path = dir.join(format!("{name}.json"));
        if path.exists() {
            let data = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, name: &str) -> Result<()> {
        let dir = Self::save_dir()?;
        let path = dir.join(format!("{name}.json"));
        let data = serde_json::to_string(self)?;
        let mut f = fs::File::create(path)?;
        write!(f, "{data}")?;
        Ok(())
    }

    fn save_dir() -> Result<Utf8PathBuf> {
        let workspace_root = MetadataCommand::new().exec()?.workspace_root;
        let dir = workspace_root.join("target/uniffi-bench/samples/");
        if !dir.exists() {
            fs::create_dir_all(&dir).with_context(|| format!("while creating dir {dir}"))?;
        }
        Ok(dir)
    }
}

impl Default for BenchmarkMeasurements {
    fn default() -> Self {
        Self {
            iterations: ITERATIONS,
            samples: HashMap::default(),
        }
    }
}

pub fn print_headers(save_name: &str, compare_save_names: &[String]) {
    print_col("benchmark", save_name, compare_save_names);
    println!("{}", "-".repeat(35 + 20 + 20 * compare_save_names.len()))
}

pub fn print_measurement(
    name: &str,
    samples: &[u64],
    saved_measurements: &[BenchmarkMeasurements],
) {
    let avg = average(samples, ITERATIONS);
    let saved = saved_measurements
        .iter()
        .map(|m| match m.samples.get(name) {
            Some(s) => {
                let saved_avg = average(s, m.iterations);
                let t = format_time(saved_avg);
                let change = (saved_avg as i64 - avg as i64) * 100 / avg as i64;
                format!("{t} ({change:+}%)")
            }
            None => "<not recorded>".to_string(),
        });
    print_col(name, format_time(avg), saved);
}

fn print_col(
    label: &str,
    current: impl Into<String>,
    compare_cols: impl IntoIterator<Item = impl Into<String>>,
) {
    let mut line = format!("{label:<35}");
    line.push_str(&format!("{:<20}", current.into()));
    for c in compare_cols {
        line.push_str(&format!("{:<20}", c.into()));
    }
    println!("{line}");
}

fn average(samples: &[u64], iterations: u64) -> u64 {
    samples.iter().sum::<u64>() / iterations / samples.len() as u64
}

fn format_time(nanoseconds: u64) -> String {
    match nanoseconds {
        0..1_000 => format!("{nanoseconds}ns"),
        1_000..10_000 => format!("{:.2}us", nanoseconds as f64 / 1_000.0),
        10_000..100_000 => format!("{:.1}us", nanoseconds as f64 / 1_000.0),
        100_000..1_000_000 => format!("{}us", nanoseconds / 1_000),
        1_000_000..10_000_000 => format!("{:.2}ms", nanoseconds as f64 / 1_000_000.0),
        10_000_000..100_000_000 => format!("{:.1}ms", nanoseconds as f64 / 1_000_000.0),
        100_000_000..1_000_000_000 => format!("{}ms", nanoseconds / 1_000_000),
        _ => format!("{:.3}s", nanoseconds as f64 / 1_000_000_000.0),
    }
}
