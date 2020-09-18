/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub mod gen_gecko_cpp;
pub use gen_gecko_cpp::{Config, Header, Source};

use super::super::interface::ComponentInterface;

pub struct Bindings {
    header: String,
    source: String,
}

/// Generate uniffi component bindings for Firefox.
///
/// ...
///
/// These files should be checked in to the Firefox source tree. The WebIDL
/// file goes in `dom/chrome-webidl`, and the header and source files can be
/// added to any directory and referenced in `moz.build`. The Rust component
/// library must also be added as a dependency to `gkrust-shared` (in
/// `toolkit/library/rust/shared`), so that the FFI symbols are linked into
/// libxul.
pub fn write_bindings(
    ci: &ComponentInterface,
    out_dir: &Path,
    _try_format_code: bool,
) -> Result<()> {
    use heck::CamelCase;

    let out_path = PathBuf::from(out_dir);

    let Bindings { header, source } = generate_bindings(&ci)?;

    let mut header_file = out_path.clone();
    header_file.push(format!("{}.h", namespace_to_file_name(ci.namespace())));
    let mut h = File::create(&header_file).context("Failed to create header file for bindings")?;
    write!(h, "{}", header)?;

    let mut source_file = out_path;
    source_file.push(format!("{}.cpp", namespace_to_file_name(ci.namespace())));
    let mut m = File::create(&source_file).context("Failed to create source file for bindings")?;
    write!(m, "{}", source)?;

    Ok(())
}

pub fn namespace_to_file_name(namespace: &str) -> String {
    use heck::CamelCase;
    namespace.to_camel_case()
}

/// Generate Gecko bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(ci: &ComponentInterface) -> Result<Bindings> {
    let config = Config::from(&ci);
    use askama::Template;
    use heck::CamelCase;

    let header = Header::new(&config, &ci)
        .render()
        .context("Failed to render header")?;

    let source = Source::new(&config, &ci)
        .render()
        .context("Failed to render source")?;

    Ok(Bindings { header, source })
}
