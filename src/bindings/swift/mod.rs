/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use anyhow::{anyhow, bail};

pub mod gen_swift;
pub use gen_swift::{BridgingHeader, Config, ModuleMap, SwiftWrapper};

use super::super::interface::ComponentInterface;

pub struct Bindings {
    header: String,
    library: String,
}

/// Generate Swift bindings for the given ComponentInterface, as a string.
pub fn generate_swift_bindings(ci: &ComponentInterface) -> Result<Bindings> {
    let config = Config::from(&ci);
    use askama::Template;
    let header = BridgingHeader::new(&config, &ci)
        .render()
        .map_err(|_| anyhow!("failed to render Swift bridging header"))?;
    let library = SwiftWrapper::new(&config, &ci)
        .render()
        .map_err(|_| anyhow!("failed to render Swift library"))?;
    Ok(Bindings { header, library })
}

fn generate_module_map(ci: &ComponentInterface, header_path: &Path) -> Result<String> {
    use askama::Template;
    let module_map = ModuleMap::new(&ci, header_path)
        .render()
        .map_err(|_| anyhow!("failed to render Swift module map"))?;
    Ok(module_map)
}

/// ...
pub fn write_swift_bindings(ci: &ComponentInterface, out_dir: &str) -> Result<()> {
    let out_path = PathBuf::from(out_dir);

    let mut header_file = out_path.clone();
    header_file.push(format!("{}-Bridging-Header.h", ci.namespace()));

    let mut module_map_file = out_path.clone();
    module_map_file.push(format!("uniffi_{}.modulemap", ci.namespace()));

    let mut library_file = out_path.clone();
    library_file.push(format!("{}.swift", ci.namespace()));

    let Bindings { header, library } = generate_swift_bindings(&ci)?;

    let mut h =
        File::create(&header_file).map_err(|e| anyhow!("Failed to create .h file: {:?}", e))?;
    write!(h, "{}", header)
        .map_err(|e| anyhow!("Failed to write Swift bridging header: {:?}", e))?;

    let mut l = File::create(&library_file)
        .map_err(|e| anyhow!("Failed to create .swift file: {:?}", e))?;
    write!(l, "{}", library).map_err(|e| anyhow!("Failed to write Swift library: {:?}", e))?;

    Ok(())
}
