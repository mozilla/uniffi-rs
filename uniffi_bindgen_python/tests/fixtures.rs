/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Context, Result};
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use uniffi_testing::UniFFITestHelper;

/// Run the test fixtures from UniFFI

fn run_test(fixture_name: &str, script_file: &str) -> Result<()> {
    let script_path = Path::new(".")
        .join("tests")
        .join(script_file)
        .canonicalize()?;
    let test_helper = UniFFITestHelper::new(fixture_name).context("UniFFITestHelper::new")?;
    let out_dir = test_helper
        .create_out_dir(env!("CARGO_TARGET_TMPDIR"), &script_path)
        .context("create_out_dir")?;
    test_helper
        .copy_cdylibs_to_out_dir(&out_dir)
        .context("copy_cdylibs_to_out_dir")?;
    generate_sources(&out_dir, &test_helper).context("generate_sources")?;

    let pythonpath = env::var_os("PYTHONPATH").unwrap_or_else(|| OsString::from(""));
    let pythonpath =
        env::join_paths(env::split_paths(&pythonpath).chain(std::iter::once(out_dir.clone())))?;

    let status = Command::new("python3")
        .current_dir(out_dir)
        .env("PYTHONPATH", pythonpath)
        .arg(script_path)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .context("Failed to spawn `python3` when running script")?
        .wait()
        .context("Failed to wait for `python3` when running script")?;
    if !status.success() {
        anyhow::bail!("running `python3` failed");
    }
    Ok(())
}

fn generate_sources(out_dir: &Path, test_helper: &UniFFITestHelper) -> Result<()> {
    for source in test_helper.get_compile_sources()? {
        let mut cmd_line = vec![
            "uniffi-bindgen-python".to_string(),
            "--out-dir".to_string(),
            out_dir.to_string_lossy().to_string(),
            "--no-format".to_string(),
        ];
        if let Some(path) = source.config_path {
            cmd_line.push("--config-path".to_string());
            cmd_line.push(path.to_string_lossy().to_string())
        }
        cmd_line.push(source.udl_path.to_string_lossy().to_string());
        uniffi_bindgen_python::run(cmd_line)?;
    }
    Ok(())
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
    (test_arithmetic,        "uniffi-example-arithmetic",   "scripts/test_arithmetic.py"),
    (test_callbacks_example, "uniffi-example-callbacks",    "scripts/test_callbacks_example.py"),
    (test_custom_types,      "uniffi-example-custom-types", "scripts/test_custom_types.py"),
    (test_geometry,          "uniffi-example-geometry",     "scripts/test_geometry.py"),
    (test_rondpoint,         "uniffi-example-rondpoint",    "scripts/test_rondpoint.py"),
    (test_sprites,           "uniffi-example-sprites",      "scripts/test_sprites.py"),
    (test_todolist,          "uniffi-example-todolist",     "scripts/test_todolist.py"),
    // Fixtures
    (test_callbacks,      "uniffi-fixture-callbacks",      "scripts/test_callbacks.py"),
    (test_chronological,  "uniffi-fixture-time",           "scripts/test_chronological.py"),
    (test_coverall,       "uniffi-fixture-coverall",       "scripts/test_coverall.py"),
    (test_external_types, "uniffi-fixture-external-types", "scripts/test_external_types.py"),
    (test_ext_types,      "uniffi-fixture-ext-types",      "scripts/test_imported_types.py"),
    (test_ext_types_guid, "uniffi-fixture-ext-types-guid", "scripts/test_guid.py"),
    // Regression tests
    (test_cdylib_crate_dependency,  "uniffi-fixture-regression-cdylib-dependency-ffi-crate",   "scripts/test_cdylib_crate_dependency.py"),
    (test_enum_without_i32_helpers, "uniffi-fixture-regression-i356-enum-without-int-helpers", "scripts/test_enum_without_i32_helpers.py"),
}
