/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Context, Result};
use std::{
    env,
    ffi::OsString,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub mod gen_kotlin;
pub use gen_kotlin::{Config, KotlinWrapper};

use super::super::interface::ComponentInterface;

pub fn write_bindings(
    config: &Config,
    ci: &ComponentInterface,
    out_dir: &Path,
    try_format_code: bool,
) -> Result<()> {
    let mut kt_file = full_bindings_path(config, out_dir)?;
    std::fs::create_dir_all(&kt_file)?;
    kt_file.push(format!("{}.kt", ci.namespace()));
    let mut f = File::create(&kt_file).context("Failed to create .kt file for bindings")?;
    write!(f, "{}", generate_bindings(config, ci)?)?;
    if try_format_code {
        if let Err(e) = Command::new("ktlint")
            .arg("-F")
            .arg(kt_file.to_str().unwrap())
            .output()
        {
            println!(
                "Warning: Unable to auto-format {} using ktlint: {:?}",
                kt_file.file_name().unwrap().to_str().unwrap(),
                e
            )
        }
    }
    Ok(())
}

fn full_bindings_path(config: &Config, out_dir: &Path) -> Result<PathBuf> {
    let package_path: PathBuf = config.package_name().split('.').collect();
    Ok(PathBuf::from(out_dir).join(package_path))
}

// Generate kotlin bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    use askama::Template;
    KotlinWrapper::new(config.clone(), ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render kotlin bindings"))
}

/// Generate kotlin bindings for the given namespace, then use the kotlin
/// command-line tools to compile them into a .jar file.
pub fn compile_bindings(config: &Config, ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let mut kt_file = full_bindings_path(config, out_dir)?;
    kt_file.push(format!("{}.kt", ci.namespace()));
    let mut jar_file = PathBuf::from(out_dir);
    jar_file.push(format!("{}.jar", ci.namespace()));
    let status = Command::new("kotlinc")
        // Our generated bindings should not produce any warnings; fail tests if they do.
        .arg("-Werror")
        // Reflect $CLASSPATH from the environment, to help find `jna.jar`.
        .arg("-classpath")
        .arg(env::var("CLASSPATH").unwrap_or_else(|_| "".to_string()))
        .arg(&kt_file)
        .args(config.extra_sources.clone().into_iter().flat_map(|packages| packages.into_iter().map(|p| (PathBuf::from(out_dir).join(p)))))
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

/// Execute the specifed kotlin script, with classpath based on the generated
// artifacts in the given output directory.
pub fn run_script(out_dir: &Path, script_file: &Path) -> Result<()> {
    let mut classpath = env::var_os("CLASSPATH").unwrap_or_else(|| OsString::from(""));
    // This lets java find the compiled library for the rust component.
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
    let mut cmd = Command::new("kotlinc");
    // Make sure it can load the .jar and its dependencies.
    cmd.arg("-classpath").arg(classpath);
    // Code that wants to use an API with unsigned types, must opt in to this experimental Kotlin feature.
    // Specify it here in order to not have to worry about that when writing tests.
    cmd.arg("-Xopt-in=kotlin.ExperimentalUnsignedTypes");
    // Enable runtime assertions, for easy testing etc.
    cmd.arg("-J-ea");
    // Our test scripts should not produce any warnings.
    cmd.arg("-Werror");
    cmd.arg("-script").arg(script_file);
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
