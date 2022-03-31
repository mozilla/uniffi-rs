/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Context, Result};
use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use uniffi_testing::UniFFITestHelper;

/// Run the test fixtures from UniFFI

fn run_test(fixture_name: &str, script_file: &str) -> Result<()> {
    let script_path = Path::new(".").join("tests").join(script_file);
    let test_helper = UniFFITestHelper::new(fixture_name).context("UniFFITestHelper::new")?;
    let out_dir = test_helper.create_out_dir(env!("CARGO_TARGET_TMPDIR"), &script_path).context("create_out_dir")?;
    test_helper.copy_cdylibs_to_out_dir(&out_dir).context("copy_cdylibs_to_out_dir")?;
    generate_sources(&out_dir, &test_helper).context("generate_sources")?;
    let jar_file = build_jar(&fixture_name, &out_dir).context("build_jar")?;

    let status = Command::new("kotlinc")
        .arg("-classpath").arg(calc_classpath(vec![&out_dir, &jar_file]))
        // Enable runtime assertions, for easy testing etc.
        .arg("-J-ea")
        // Our test scripts should not produce any warnings.
        .arg("-Werror")
        .arg("-script").arg(script_path)
        .spawn()
        .context("Failed to spawn `kotlinc` to run Kotlin script")?
        .wait()
        .context("Failed to wait for `kotlinc` when running Kotlin script")?;
    if !status.success() {
        anyhow::bail!("running `kotlinc` failed")
    }
    Ok(())
}

fn generate_sources(out_dir: &Path, test_helper: &UniFFITestHelper) -> Result<()> {
    for source in test_helper.get_compile_sources()? {
        let mut cmd_line = vec!["uniffi-bindgen-kotlin".to_string(), "--out-dir".to_string(), out_dir.to_string_lossy().to_string()];
        if let Some(path) = source.config_path {
            cmd_line.push("--config-path".to_string());
            cmd_line.push(path.to_string_lossy().to_string())
        }
        cmd_line.push(source.udl_path.to_string_lossy().to_string());
        println!("{:?}", cmd_line);
        uniffi_bindgen_kotlin::run(cmd_line)?;
    }
    Ok(())
}

/// Generate kotlin bindings for the given namespace, then use the kotlin
/// command-line tools to compile them into a .jar file.
fn build_jar(fixture_name: &str, out_dir: &Path) -> Result<PathBuf> {
    let mut jar_file = PathBuf::from(out_dir);
    jar_file.push(format!("{}.jar", fixture_name));

    let status = Command::new("kotlinc")
        // Our generated bindings should not produce any warnings; fail tests if they do.
        .arg("-Werror")
        .arg("-d")
        .arg(&jar_file)
        .arg("-classpath")
        .arg(calc_classpath(vec![]))
        .args(
            glob::glob(&out_dir.join("*.kt").to_string_lossy())?.flatten().map(|p| String::from(p.to_string_lossy()))
        )
        .spawn()
        .context("Failed to spawn `kotlinc` to compile the bindings")?
        .wait()
        .context("Failed to wait for `kotlinc` when compiling the bindings")?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(jar_file)
}

fn calc_classpath(extra_paths: Vec<&Path>) -> String {
    extra_paths
        .into_iter()
        .map(|p| p.to_string_lossy())
        // Add the system classpath as a component, using the fact that env::var returns an Option,
        // which implement Iterator
        .chain(env::var("CLASSPATH").map(Cow::from))
        .collect::<Vec<Cow<str>>>()
        .join(":")
}

macro_rules! fixture_tests {
    {
        $(($test_name:ident, $fixture_name:expr, $test_script:expr),)*
    } => {
    $(
        #[test]
        fn $test_name() -> Result<()> {
            run_test($fixture_name, $test_script)
        }
    )*
    }
}

fixture_tests! {
    // Examples
    (test_arithmetic,        "uniffi-example-arithmetic",   "scripts/test_arithmetic.kts"),
    (test_callbacks_example, "uniffi-example-callbacks",    "scripts/test_callbacks_example.kts"),
    (test_custom_types,      "uniffi-example-custom-types", "scripts/test_custom_types.kts"),
    (test_geometry,          "uniffi-example-geometry",     "scripts/test_geometry.kts"),
    (test_rondpoint,         "uniffi-example-rondpoint",    "scripts/test_rondpoint.kts"),
    (test_sprites,           "uniffi-example-sprites",      "scripts/test_sprites.kts"),
    (test_todolist,          "uniffi-example-todolist",     "scripts/test_todolist.kts"),
    // Fixtures
    (test_callbacks,           "uniffi-fixture-callbacks",    "scripts/test_callbacks.kts"),
    (test_chronological,       "uniffi-fixture-time",         "scripts/test_chronological.kts"),
    (test_coverall,            "uniffi-fixture-coverall",     "scripts/test_coverall.kts"),
    (test_coverall_handlerace, "uniffi-fixture-coverall",     "scripts/test_coverall_handlerace.kts"),
    (test_external_types,      "uniffi-fixture-ext-types",    "scripts/test_imported_types.kts"),
    // Regression tests
    (test_cdylib_crate_dependency,     "uniffi-fixture-regression-cdylib-dependency-ffi-crate",        "scripts/test_cdylib_crate_dependency.kts"),
    (test_enum_without_i32_helpers,    "uniffi-fixture-regression-i356-enum-without-int-helpers",      "scripts/test_enum_without_i32_helpers.kts"),
    (test_experimental_unsigned_types, "uniffi-fixture-regression-kotlin-experimental-unsigned-types", "scripts/test_experimental_unsigned_types.kts"),
}
