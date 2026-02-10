/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    bail,
    bindings::{generate, GenerateOptions, RunScriptOptions, TargetLanguage},
    Context, Result,
};
use camino::{Utf8Path, Utf8PathBuf};
use std::env;
use std::process::Command;
use uniffi_testing::UniFFITestHelper;

/// Run Kotlin tests for a UniFFI test fixture
pub fn run_test(tmp_dir: &str, fixture_name: &str, script_file: &str) -> Result<()> {
    run_script(
        tmp_dir,
        fixture_name,
        script_file,
        vec![],
        &RunScriptOptions::default(),
    )
}

/// Run a Kotlin script
///
/// This function will set things up so that the script can import the UniFFI bindings for a crate
pub fn run_script(
    tmp_dir: &str,
    crate_name: &str,
    script_file: &str,
    args: Vec<String>,
    options: &RunScriptOptions,
) -> Result<()> {
    let script_path = Utf8Path::new(script_file);
    let test_helper = UniFFITestHelper::new(crate_name)?;
    let out_dir = test_helper.create_out_dir(tmp_dir, script_path)?;
    let cdylib_path = test_helper.copy_cdylib_to_out_dir(&out_dir)?;

    generate(GenerateOptions {
        languages: vec![TargetLanguage::Kotlin],
        source: cdylib_path.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        ..GenerateOptions::default()
    })?;
    let jar_file = build_jar(crate_name, &out_dir, options)?;

    let mut command = kotlinc_command(options);
    command
        .arg("-classpath")
        .arg(calc_classpath(vec![&out_dir, &jar_file]))
        // Enable runtime assertions, for easy testing etc.
        .arg("-J-ea")
        // Our test scripts should not produce any warnings.
        .arg("-Werror")
        .arg("-script")
        .arg(script_path)
        .args(if args.is_empty() {
            vec![]
        } else {
            std::iter::once(String::from("--")).chain(args).collect()
        });

    let status = command
        .spawn()
        .context("Failed to spawn `kotlinc` to run Kotlin script")?
        .wait()
        .context("Failed to wait for `kotlinc` when running Kotlin script")?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(())
}

/// Generate kotlin bindings for the given namespace, then use the kotlin
/// command-line tools to compile them into a .jar file.
fn build_jar(
    crate_name: &str,
    out_dir: &Utf8Path,
    options: &RunScriptOptions,
) -> Result<Utf8PathBuf> {
    let mut jar_file = Utf8PathBuf::from(out_dir);
    jar_file.push(format!("{crate_name}.jar"));
    let sources = glob::glob(out_dir.join("**/*.kt").as_str())?
        .flatten()
        .map(|p| String::from(p.to_string_lossy()))
        .collect::<Vec<String>>();
    if sources.is_empty() {
        bail!("No kotlin sources found in {out_dir}")
    }

    let installation = find_kotlinc_installation()?;
    // kotlin-serialization-compiler-plugin.jar (no x after kotlin!) was not present until Kotlin 1.9.
    let kotlinx_serialization_plugin =
        installation.join("lib/kotlinx-serialization-compiler-plugin.jar");

    let mut command = kotlinc_command(options);
    command
        .arg(format!("-Xplugin={kotlinx_serialization_plugin}"))
        // Our generated bindings should not produce any warnings; fail tests if they do.
        .arg("-Werror")
        .arg("-d")
        .arg(&jar_file)
        .arg("-classpath")
        .arg(calc_classpath(vec![]))
        .args(sources);

    let status = command
        .spawn()
        .context("Failed to spawn `kotlinc` to compile the bindings")?
        .wait()
        .context("Failed to wait for `kotlinc` when compiling the bindings")?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(jar_file)
}

fn kotlinc_command(options: &RunScriptOptions) -> Command {
    let mut command = Command::new("kotlinc");
    if !options.show_compiler_messages {
        command.arg("-nowarn");
    }
    command
}

fn find_kotlinc_installation() -> Result<Utf8PathBuf> {
    let Some(path_var) = env::var_os("PATH") else {
        bail!("Environment variable PATH not present")
    };
    let kotlinc_executable_name = if cfg!(target_os = "windows") {
        "kotlinc.bat"
    } else {
        "kotlinc"
    };
    if let Some(installation) = env::split_paths(&path_var)
        .flat_map(|path| {
            // If we find <path>/kotlinc or <path>/kotlinc.bat,
            let compiler_path = path.join(kotlinc_executable_name);
            compiler_path
                .try_exists()
                .is_ok_and(|e| e)
                .then_some((path, compiler_path))
        })
        .flat_map(|(path, compiler_path)| {
            // Scan <path> and the parent of the symbolic link target of <path>/kotlinc,
            let link_resolved_path = compiler_path
                .canonicalize()
                .ok()
                .and_then(|p| p.parent().map(ToOwned::to_owned));
            std::iter::once(path).chain(link_resolved_path)
        })
        .filter(|path| {
            // Make sure <path> is <installation>/bin,
            path.file_name()
                .and_then(|f| f.to_str())
                .is_some_and(|s| s == "bin")
        })
        .flat_map(|path| {
            // Collect <installation>,
            path.parent().map(ToOwned::to_owned)
        })
        .filter(|installation| {
            // And if <installation>/lib exists,
            installation.join("lib").try_exists().is_ok_and(|e| e)
        })
        // Try converting it into Utf8PathBuf and,
        .flat_map(Utf8PathBuf::from_path_buf)
        .next()
    {
        // Return the first occurrence of such <installation>.
        return Ok(installation);
    }

    bail!("Could not find a directory containing the kotlinc executable and the required JAR files")
}

fn calc_classpath(extra_paths: Vec<&Utf8Path>) -> String {
    extra_paths
        .into_iter()
        .map(|p| p.to_string())
        // Add the system classpath as a component, using the fact that env::var returns an Option,
        // which implement Iterator
        .chain(env::var("CLASSPATH"))
        .collect::<Vec<String>>()
        .join(":")
}
