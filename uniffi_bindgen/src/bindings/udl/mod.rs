/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub mod gen_udl;
pub use gen_udl::UDLFile;

use super::super::interface::ComponentInterface;

// Generate UDL bindings for the given ComponentInterface, in the given output directory.

pub fn write_bindings(ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let mut udl_file = PathBuf::from(out_dir);
    udl_file.push(format!("{}.generated.udl", ci.namespace()));
    println!("writing to {:?}", udl_file);
    let mut f =
        File::create(&udl_file).context("Failed to create .generated.udl file for bindings")?;
    write!(f, "{}", generate_udl_bindings(&ci)?)?;
    Ok(())
}

// Generate UDL bindings for the given ComponentInterface, as a string.

pub fn generate_udl_bindings(ci: &ComponentInterface) -> Result<String> {
    use askama::Template;
    UDLFile::new(&ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render UDL bindings"))
}
