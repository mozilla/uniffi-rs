/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    ffi::OsString,
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};

pub mod gen_swift;
pub use gen_swift::{BridgingHeader, Config, ModuleMap, SwiftWrapper};

use super::super::interface::ComponentInterface;

pub struct Bindings {
    header: String,
    library: String,
}

/// Generate uniffi component bindings for swift.
///
/// Unlike other target languages, binding to rust code from swift involves more than just
/// generating a `.swift` file. We also need to produce a `.h` file with the C-level API
/// declarations, and a `.modulemap` file to tell swift how to use it.
///
/// Further complicating matters is that fact that swift doesn't seem to like having multiple
/// `.modulemap` files referring to headers in the same directory (if you try it, it will complain
/// about multiple umbrella headers covering the same directory). That's not a problem for
/// stand-alone crates but it doesn't work well when you have multiple uniffi components defined
/// in the same workspace.
///
/// We work around this by generating the module-mapping instructions in a separate subdirectory
/// for each module, producing a directory structure that looks like this:
///
///    libuniffi_example.dylib
///    example.swift
///    example.swiftmodule-dir/
///        example-Bridging-Header.h
///        uniffi.modulemap
///
/// It's not very elegant, and the details will probably change as we get more experience with
/// building independent swift modules. But it does the job for now.
pub fn write_bindings(ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let out_path = PathBuf::from(out_dir);

    let mut module_dir = out_path.clone();
    module_dir.push(format!("{}.swiftmodule-dir", ci.namespace()));
    fs::create_dir_all(&module_dir)?;

    let mut module_map_file = module_dir.clone();
    module_map_file.push("uniffi.modulemap");

    let mut header_file = module_dir;
    header_file.push(format!("{}-Bridging-Header.h", ci.namespace()));

    let mut source_file = out_path;
    source_file.push(format!("{}.swift", ci.namespace()));

    let Bindings { header, library } = generate_bindings(&ci)?;

    let mut h = File::create(&header_file).context("Failed to create .h file for bindings")?;
    write!(h, "{}", header)?;

    let mut m =
        File::create(&module_map_file).context("Failed to create .modulemap file for bindings")?;
    write!(m, "{}", generate_module_map(&ci, &header_file)?)?;

    let mut l = File::create(&source_file).context("Failed to create .swift file for bindings")?;
    write!(l, "{}", library)?;

    Ok(())
}

/// Generate Swift bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(ci: &ComponentInterface) -> Result<Bindings> {
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
pub fn compile_bindings(ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let out_path = PathBuf::from(out_dir);

    let mut module_map_file = out_path.clone();
    module_map_file.push(format!("{}.swiftmodule-dir", ci.namespace()));
    module_map_file.push("uniffi.modulemap");

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
        .context("Failed to spawn `swiftc` when compiling bindings")?
        .wait()
        .context("Failed to wait for `swiftc` when compiling bindings")?;
    if !status.success() {
        bail!("running `swiftc` failed")
    }
    Ok(())
}

pub fn run_script(out_dir: Option<&Path>, script_file: Option<&Path>) -> Result<()> {
    let mut cmd = std::process::Command::new("swift");

    // Find any module maps and/or dylibs in the target directory, and tell swift to use them.
    if let Some(out_dir) = out_dir {
        cmd.arg("-I").arg(out_dir).arg("-L").arg(out_dir);
        for entry in PathBuf::from(out_dir)
            .read_dir()
            .context("Failed to list target directory when running script")?
        {
            let entry = entry.context("Failed to list target directory when running script")?;
            if let Some(ext) = entry.path().extension() {
                if ext == "swiftmodule-dir" {
                    let mut module_map_file = entry.path();
                    module_map_file.push("uniffi.modulemap");
                    let mut option = OsString::from("-fmodule-map-file=");
                    option.push(module_map_file);
                    cmd.arg("-Xcc");
                    cmd.arg(option);
                } else if ext == "dylib" || ext == "so" {
                    let mut option = OsString::from("-l");
                    option.push(entry.path());
                    cmd.arg(option);
                }
            }
        }
    }

    if let Some(script) = script_file {
        cmd.arg(script);
    }
    let status = cmd
        .spawn()
        .context("Failed to spawn `swift` when running script")?
        .wait()
        .context("Failed to wait for `swift` when running script")?;
    if !status.success() {
        bail!("running `swift` failed")
    }
    Ok(())
}
