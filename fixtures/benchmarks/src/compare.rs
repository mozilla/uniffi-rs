/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    collections::{BTreeSet, HashMap},
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    time::SystemTime,
};

use anyhow::{anyhow, bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;

pub struct CriterionMeasurementTracker {
    workspace_dir: Utf8PathBuf,
    initial_mtimes: HashMap<Utf8PathBuf, SystemTime>,
}

impl CriterionMeasurementTracker {
    pub fn new() -> Result<Self> {
        let mut initial_mtimes = HashMap::new();
        let workspace_dir = find_workspace_root()?;
        for test_case in test_case_dirs(&workspace_dir)? {
            let path = estimate_path(&test_case);
            if path.exists() {
                let meta = path
                    .metadata()
                    .with_context(|| format!("Reading metadata for {path}"))?;
                initial_mtimes.insert(path, meta.modified()?);
            }
        }
        Ok(Self {
            workspace_dir,
            initial_mtimes,
        })
    }

    pub fn save(&self, name: &str) -> Result<()> {
        let uniffi_bench = self.workspace_dir.join("target/uniffi-bench");
        if !uniffi_bench.exists() {
            create_dir_all(&uniffi_bench)?;
        }
        let path = uniffi_bench.join(format!("{name}.json"));
        let contents = serde_json::to_string(&self.times_from_last_run()?)?;
        let mut f = File::create(path)?;
        write!(f, "{contents}")?;
        Ok(())
    }

    fn read(&self, name: &str) -> Result<HashMap<String, u64>> {
        let uniffi_bench = self.workspace_dir.join("target/uniffi-bench");
        if !uniffi_bench.exists() {
            create_dir_all(&uniffi_bench)?;
        }
        let path = uniffi_bench.join(format!("{name}.json"));
        if !path.exists() {
            bail!("No save found for {name}");
        }
        let contents = read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn compare(&self, compare: &[String], save_name: Option<&str>) -> Result<()> {
        let mut all_names = Vec::from_iter(compare.iter().cloned());
        match save_name {
            Some(s) if !all_names.iter().any(|s2| s == s2) => all_names.push(s.to_string()),
            _ => (),
        };
        let all_times = all_names
            .iter()
            .map(|name| self.read(name))
            .collect::<Result<Vec<HashMap<String, u64>>>>()?;
        let all_cases = BTreeSet::from_iter(all_times.iter().flat_map(|map| map.keys()));
        print_col("", &all_names);
        println!("{}", "-".repeat(35 + 20 * all_names.len()));
        for case in all_cases {
            let mut first_time = None;
            let times = all_times.iter().map(|map| match map.get(case.as_str()) {
                Some(nanos) => {
                    let nanos = *nanos;
                    let time_str = format_time(nanos);
                    match first_time {
                        None => {
                            first_time = Some(nanos);
                            time_str
                        }
                        Some(first_time) => {
                            //let percent_change = format!("{}", (first_time - nanos) as f64 * 100.0 / first_time as f64);
                            let percent_change =
                                format!("({:.01}x)", first_time as f64 / nanos as f64);
                            format!("{time_str} {percent_change:>7}")
                        }
                    }
                }
                None => "".to_string(),
            });
            print_col(case, times);
        }
        Ok(())
    }

    fn times_from_last_run(&self) -> Result<HashMap<String, u64>> {
        let mut times = HashMap::new();
        for dir in test_case_dirs(&self.workspace_dir)? {
            let test_case = dir.file_name().expect("invalid workspace dir");
            let path = estimate_path(&dir);
            if path.exists() && self.path_changed(&path)? {
                times.insert(test_case.to_string(), read_time(&path)?);
            }
        }
        Ok(times)
    }

    fn path_changed(&self, path: &Utf8Path) -> Result<bool> {
        let meta = path
            .metadata()
            .with_context(|| format!("Reading metadata for {path}"))?;
        let mtime = meta.modified()?;
        Ok(match self.initial_mtimes.get(path) {
            None => true,
            Some(initial_mtime) => mtime != *initial_mtime,
        })
    }
}

fn test_case_dirs(workspace_dir: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    let criterion_dir = workspace_dir.join("target/criterion");
    let mut dirs = vec![];
    if criterion_dir.exists() {
        for entry in criterion_dir.read_dir()? {
            dirs.push(utf8_path_buf(entry?.path())?);
        }
    }

    Ok(dirs)
}

fn estimate_path(test_case_dir: &Utf8Path) -> Utf8PathBuf {
    // Get estimates from the `new` directory, which represents the benchmarks created by
    // this run.
    test_case_dir.join("new/estimates.json")
}

fn find_workspace_root() -> Result<Utf8PathBuf> {
    let current_dir = utf8_path_buf(std::env::current_dir()?)?;

    let mut dir = current_dir.as_path();

    loop {
        if dir.join("Cargo.toml").exists()
            && dir.join("target").exists()
            && dir.join("LICENSE").exists()
        {
            return Ok(dir.to_path_buf());
        }
        dir = match dir.parent() {
            Some(dir) => dir,
            None => bail!("Can't find workspace root dir"),
        };
    }
}

fn read_time(estimates_json_path: &Utf8Path) -> Result<u64> {
    let data = read_to_string(estimates_json_path)?;
    let json: Value = serde_json::from_str(&data)?;
    match json
        .pointer("/median/point_estimate")
        .and_then(Value::as_f64)
    {
        None => bail!("Can't find median value in {estimates_json_path}"),
        Some(v) => Ok(v as u64),
    }
}

fn utf8_path_buf(path_buf: std::path::PathBuf) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path_buf).map_err(|p| anyhow!("Non-UTF8 path: {p:?}"))
}

fn print_col(label: &str, columns: impl IntoIterator<Item = impl Into<String>>) {
    let mut line = format!("{label:<35}");
    for c in columns {
        line.push_str(&format!("{:>20}", c.into()));
    }
    println!("{line}");
}

fn format_time(nanoseconds: u64) -> String {
    match nanoseconds {
        0..1_000 => format!("{nanoseconds}ns"),
        1_000..1_000_000 => format!("{:.2}us", nanoseconds as f64 / 1_000.0),
        1_000_000..1_000_000_000 => format!("{:.2}ms", nanoseconds as f64 / 1_000_000.0),
        _ => format!("{:.2}s ", nanoseconds as f64 / 1_000_000_000.0),
    }
}
