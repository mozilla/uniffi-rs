/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{anyhow, bail, Context, Result};
use std::{
    ffi::OsString,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub mod gen_gecko;
pub use gen_gecko::{Config, GeckoWrapper, Header, WebIdl};

use super::super::interface::ComponentInterface;

pub struct Bindings {
    header: String,
    webidl: String,
    library: String,
}

/// Generate uniffi component bindings for Gecko.
///
/// Bindings to a Rust interface for Gecko involves more than just generating a
/// `.cpp` file. We also need to produce a `.h` file with the C-level API
/// declarations and a `.webidl` file with the interface declaration.
pub fn write_bindings(
    ci: &ComponentInterface,
    out_dir: &Path,
    _try_format_code: bool,
) -> Result<()> {
    let out_path = PathBuf::from(out_dir);

    let mut header_file = out_path.clone();
    header_file.push(format!("{}.h", ci.namespace()));

    let mut webidl_file = out_path.clone();
    webidl_file.push(format!("{}.webidl", ci.namespace()));

    let mut source_file = out_path;
    source_file.push(format!("{}.cpp", ci.namespace()));

    let Bindings {
        header,
        webidl,
        library,
    } = generate_bindings(&ci)?;

    let mut h = File::create(&header_file).context("Failed to create .h file for bindings")?;
    write!(h, "{}", header)?;

    let mut w = File::create(&webidl_file).context("Failed to create .webidl file for bindings")?;
    write!(w, "{}", webidl)?;

    let mut l = File::create(&source_file).context("Failed to create .cpp file for bindings")?;
    write!(l, "{}", library)?;

    Ok(())
}

/// Generate Gecko bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(ci: &ComponentInterface) -> Result<Bindings> {
    let config = Config::from(&ci);
    use askama::Template;
    let header = Header::new(&config, &ci)
        .render()
        .context("Failed to render Gecko header")?;
    let webidl = WebIdl::new(&config, &ci)
        .render()
        .context("Failed to render WebIDL bindings")?;
    let library = GeckoWrapper::new(&config, &ci)
        .render()
        .context("Failed to render Gecko library")?;
    Ok(Bindings {
        header,
        webidl,
        library,
    })
}
