/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    ffi::{OsStr, OsString},
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
pub fn write_swift_bindings(ci: &ComponentInterface, out_dir: &OsStr) -> Result<()> {
    let out_path = PathBuf::from(out_dir);

    let mut header_file = out_path.clone();
    header_file.push(format!("{}-Bridging-Header.h", ci.namespace()));

    let mut module_map_file = out_path.clone();
    module_map_file.push(format!("uniffi_{}.modulemap", ci.namespace()));

    let mut source_file = out_path;
    source_file.push(format!("{}.swift", ci.namespace()));

    let Bindings { header, library } = generate_swift_bindings(&ci)?;

    let mut h =
        File::create(&header_file).map_err(|e| anyhow!("Failed to create .h file: {:?}", e))?;
    write!(h, "{}", header)
        .map_err(|e| anyhow!("Failed to write Swift bridging header: {:?}", e))?;

    let mut m = File::create(&module_map_file)
        .map_err(|e| anyhow!("Failed to create .modulemap file: {:?}", e))?;
    write!(m, "{}", generate_module_map(&ci, &header_file)?)
        .map_err(|e| anyhow!("Failed to write Swift module map: {:?}", e))?;

    let mut l =
        File::create(&source_file).map_err(|e| anyhow!("Failed to create .swift file: {:?}", e))?;
    write!(l, "{}", library).map_err(|e| anyhow!("Failed to write Swift library: {:?}", e))?;

    Ok(())
}

/// ...
pub fn compile_swift_module(ci: &ComponentInterface, out_dir: &OsStr) -> Result<()> {
    let out_path = PathBuf::from(out_dir);

    let mut module_map_file = out_path.clone();
    module_map_file.push(format!("uniffi_{}.modulemap", ci.namespace()));

    let mut module_map_file_option = OsString::from("-fmodule-map-file=");
    module_map_file_option.push(module_map_file.as_os_str());

    let mut source_file = out_path.clone();
    source_file.push(format!("{}.swift", ci.namespace()));

    let mut dylib_file = out_path.clone();
    dylib_file.push(format!("lib{}.dylib", ci.namespace()));

    // `-emit-library -o <path>` generates a `.dylib`, so that we can use the
    // Swift module from the REPL. Otherwise, we'll get "Couldn't lookup
    // symbols" when we try to import the module.
    // See https://bugs.swift.org/browse/SR-1191.

    let status = std::process::Command::new("swiftc")
        .arg("-module-name")
        .arg(ci.namespace())
        .arg("-emit-library")
        .arg("-o")
        .arg(&dylib_file)
        .arg("-emit-module")
        .arg("-emit-module-path")
        .arg(&out_path)
        .arg("-parse-as-library")
        .arg("-L")
        .arg(&out_path)
        .arg(format!("-luniffi_{}", ci.namespace()))
        .arg("-Xcc")
        .arg(module_map_file_option)
        .arg(source_file)
        .spawn()
        .map_err(|_| anyhow::anyhow!("failed to spawn `swiftc`"))?
        .wait()
        .map_err(|_| anyhow::anyhow!("failed to wait for `swiftc` subprocess"))?;
    if !status.success() {
        bail!("running `swiftc` failed")
    }
    Ok(())
}

pub fn run_swift_script(out_dir: Option<&OsStr>, script_file: Option<&str>) -> Result<()> {
    // TODO: Don't hard-code library names for the REPL.
    const LIBS: &[&str] = &["-larithmetic", "-lgeometry", "-lsprites"];
    const MODULE_MAPS: &[&str] = &[
        "uniffi_arithmetic.modulemap",
        "uniffi_geometry.modulemap",
        "uniffi_sprites.modulemap",
    ];

    let mut cmd = std::process::Command::new("swift");
    cmd.args(LIBS);

    if let Some(out_dir) = out_dir {
        let out_path = PathBuf::from(out_dir);
        let cc_options: Vec<OsString> = MODULE_MAPS
            .iter()
            .flat_map(|module_map| {
                let mut module_map_file = out_path.clone();
                module_map_file.push(module_map);
                if !module_map_file.exists() {
                    // Missing module maps (for example, `uniffi_arithmetic` when we're
                    // running the `geometry` example) will fail, so ignore maps that
                    // don't exist. Gross!
                    return Vec::new();
                }

                let mut option = OsString::from("-fmodule-map-file=");
                option.push(module_map_file.as_os_str());

                vec![OsString::from("-Xcc"), option]
            })
            .collect();

        cmd.arg("-I")
            .arg(out_dir)
            .arg("-L")
            .arg(out_dir)
            .args(cc_options);
    }

    if let Some(script) = script_file {
        cmd.arg(script);
    }

    let status = cmd
        .spawn()
        .map_err(|_| anyhow::anyhow!("failed to spawn `swift`"))?
        .wait()
        .map_err(|_| anyhow::anyhow!("failed to wait for `swift` subprocess"))?;
    if !status.success() {
        bail!("running `swift` failed")
    }

    Ok(())
}
