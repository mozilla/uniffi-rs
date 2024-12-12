/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Support for the `peek` and `diff` CLI subcommands

use std::{fs, io::Write, process::Command};

use anyhow::Result;
use camino::Utf8Path;

#[cfg(feature = "cargo-metadata")]
pub fn diff_dir_from_cargo_metadata(target: impl ToString) -> Result<camino::Utf8PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new().exec()?;
    Ok(metadata
        .target_directory
        .join("diff")
        .join(target.to_string()))
}

pub fn peek(items: impl IntoIterator<Item = (String, String)>) {
    for (name, contents) in items.into_iter() {
        println!("-------------------- {name} --------------------");
        println!("{contents}");
        println!();
    }
}

pub fn save_diff(
    diff_dir: &Utf8Path,
    items: impl IntoIterator<Item = (String, String)>,
) -> Result<()> {
    let out_dir = diff_dir.join("old");
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    }

    for (name, contents) in items.into_iter() {
        let mut file = fs::File::create(out_dir.join(name))?;
        write!(file, "{contents}")?;
    }
    Ok(())
}

pub fn diff(diff_dir: &Utf8Path, items: impl IntoIterator<Item = (String, String)>) -> Result<()> {
    let old_out_dir = diff_dir.join("old");
    let out_dir = diff_dir.join("new");
    if !old_out_dir.exists() {
        fs::create_dir_all(&old_out_dir)?;
    }
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    }

    for (name, contents) in items.into_iter() {
        let mut file = fs::File::create(out_dir.join(name))?;
        write!(file, "{contents}")?;
    }
    Command::new("diff")
        .args(["-dur", "old", "new", "--color=auto"])
        .current_dir(diff_dir)
        .spawn()?
        .wait()?;

    Ok(())
}
