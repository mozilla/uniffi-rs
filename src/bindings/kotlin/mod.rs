/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::Write,
    path::PathBuf,
};

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub mod gen_kotlin;
pub use gen_kotlin::{Config, KotlinWrapper};

use super::super::interface::ComponentInterface;

// Generate kotlin bindings for the given ComponentInterface, as a string.

pub fn generate_kotlin_bindings(ci: &ComponentInterface) -> Result<String> {
    let config = Config::from(&ci);
    use askama::Template;
    KotlinWrapper::new(config, &ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render kotlin bindings"))
}

pub fn write_kotlin_bindings(ci: &ComponentInterface, out_dir: &OsStr) -> Result<()> {
    let mut kt_file = PathBuf::from(out_dir);
    kt_file.push(format!("{}.kt", ci.namespace()));
    let mut f =
        File::create(&kt_file).map_err(|e| anyhow!("Failed to create .kt file: {:?}", e))?;
    write!(f, "{}", generate_kotlin_bindings(&ci)?)
        .map_err(|e| anyhow!("Failed to write kotlin bindings: {:?}", e))?;
    Ok(())
}

// Generate kotlin bindings for the given ComponentInterface, then use the kotlin
// command-line tools to compile them into a .jar file.

pub fn compile_kotlin_bindings(ci: &ComponentInterface, out_dir: &OsStr) -> Result<()> {
    let mut kt_file = PathBuf::from(out_dir);
    kt_file.push(format!("{}.kt", ci.namespace()));
    let mut f =
        File::create(&kt_file).map_err(|e| anyhow!("Failed to create .kt file: {:?}", e))?;
    write!(f, "{}", generate_kotlin_bindings(&ci)?)
        .map_err(|e| anyhow!("Failed to write kotlin bindings: {:?}", e))?;
    let mut jar_file = PathBuf::from(out_dir);
    jar_file.push(format!("{}.jar", ci.namespace()));
    let status = std::process::Command::new("kotlinc")
        .arg("-classpath")
        .arg(std::env::var("CLASSPATH").unwrap_or_else(|_| "".to_string()))
        .arg(&kt_file)
        .arg("-d")
        .arg(jar_file)
        .spawn()
        .map_err(|_| anyhow::anyhow!("failed to spawn `kotlinc`"))?
        .wait()
        .map_err(|_| anyhow::anyhow!("failed to wait for `kotlinc` subprocess"))?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(())
}

// Execute the specifed kotlin script, with classpath based on the generated
// artifacts in the given output directory.

pub fn run_kotlin_script(out_dir: Option<&OsStr>, script_file: Option<&str>) -> Result<()> {
    let mut classpath = std::env::var_os("CLASSPATH").unwrap_or_else(|| OsString::from(""));
    // This lets java find the compiled library for the rust component.
    if let Some(out_dir) = out_dir {
        classpath.push(":");
        classpath.push(out_dir);
        // This lets java use any generate .jar files containing bindings for the rust component.
        for entry in PathBuf::from(out_dir)
            .read_dir()
            .map_err(|_| anyhow!("failed to read directory {:?}", out_dir))?
        {
            if let Ok(entry) = entry {
                if let Some(ext) = entry.path().extension() {
                    if ext == "jar" {
                        classpath.push(":");
                        classpath.push(entry.path().to_str().unwrap());
                    }
                }
            } else {
                bail!("error while reading directory")
            }
        }
    }
    let mut cmd = std::process::Command::new("kotlinc");
    cmd.arg("-classpath").arg(classpath);
    if let Some(script) = script_file {
        cmd.arg("-script").arg(script);
    }
    let status = cmd
        .spawn()
        .map_err(|_| anyhow::anyhow!("failed to spawn `kotlinc`"))?
        .wait()
        .map_err(|_| anyhow::anyhow!("failed to wait for `kotlinc` subprocess"))?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(())
}
