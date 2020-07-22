/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    ffi::OsString,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

pub mod gen_kotlin;
pub use gen_kotlin::{Config, KotlinWrapper};

use super::super::interface::ComponentInterface;

pub fn write_bindings(ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let mut kt_file = PathBuf::from(out_dir);
    kt_file.push(format!("{}.kt", ci.namespace()));
    let mut f = File::create(&kt_file).context("Failed to create .kt file for bindings")?;
    write!(f, "{}", generate_bindings(&ci)?)?;
    Ok(())
}

// Generate kotlin bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(ci: &ComponentInterface) -> Result<String> {
    let config = Config::from(&ci);
    use askama::Template;
    KotlinWrapper::new(config, &ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render kotlin bindings"))
}

// Generate kotlin bindings for the given ComponentInterface, then use the kotlin
// command-line tools to compile them into a .jar file.

pub fn compile_bindings(ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let mut kt_file = PathBuf::from(out_dir);
    kt_file.push(format!("{}.kt", ci.namespace()));
    let mut jar_file = PathBuf::from(out_dir);
    jar_file.push(format!("{}.jar", ci.namespace()));
    let status = std::process::Command::new("kotlinc")
        .arg("-classpath")
        .arg(std::env::var("CLASSPATH").unwrap_or_else(|_| "".to_string()))
        .arg(&kt_file)
        .arg("-d")
        .arg(jar_file)
        .spawn()
        .context("Failed to spawn `kotlinc` to compile the bindings")?
        .wait()
        .context("Failed to wait for `kotlinc` when compiling the bindings")?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(())
}

// Execute the specifed kotlin script, with classpath based on the generated
// artifacts in the given output directory.

pub fn run_script(out_dir: Option<&Path>, script_file: Option<&Path>) -> Result<()> {
    let mut classpath = std::env::var_os("CLASSPATH").unwrap_or_else(|| OsString::from(""));
    // This lets java find the compiled library for the rust component.
    if let Some(out_dir) = out_dir {
        classpath.push(":");
        classpath.push(out_dir);
        // This lets java use any generate .jar files containing bindings for the rust component.
        for entry in PathBuf::from(out_dir)
            .read_dir()
            .context("Failed to list target directory when running Kotlin script")?
        {
            let entry = entry.context("Directory listing failed while running Kotlin script")?;
            if let Some(ext) = entry.path().extension() {
                if ext == "jar" {
                    classpath.push(":");
                    classpath.push(entry.path());
                }
            }
        }
    }
    let mut cmd = std::process::Command::new("kotlinc");
    // Make sure it can load the .jar and its dependencies.
    cmd.arg("-classpath").arg(classpath);
    // Enable runtime assertions, for easy testing etc.
    cmd.arg("-J-ea");
    if let Some(script) = script_file {
        cmd.arg("-script").arg(script);
    }
    let status = cmd
        .spawn()
        .context("Failed to spawn `kotlinc` to run Kotlin script")?
        .wait()
        .context("Failed to wait for `kotlinc` when running Kotlin script")?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(())
}
