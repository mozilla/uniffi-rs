/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Context, Result};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::ffi::OsString;
use uniffi_testing::UniFFITestHelper;

/// Run the test fixtures from UniFFI

fn run_test(fixture_name: &str, script_file: &str) -> Result<()> {
    let script_path = Path::new(".").join("tests").join(script_file).canonicalize()?;
    let test_helper = UniFFITestHelper::new(fixture_name).context("UniFFITestHelper::new")?;
    let out_dir = test_helper.create_out_dir(env!("CARGO_TARGET_TMPDIR"), &script_path).context("create_out_dir")?;
    test_helper.copy_cdylibs_to_out_dir(&out_dir).context("copy_cdylibs_to_out_dir")?;
    generate_sources(&out_dir, &test_helper).context("generate_sources")?;

    let rubypath = env::var_os("RUBYLIB").unwrap_or_else(|| OsString::from(""));
    let rubypath = env::join_paths(
        env::split_paths(&rubypath).chain(vec![out_dir.to_path_buf()])
    )?;

    let status = Command::new("ruby")
        .current_dir(out_dir)
        .env("RUBYLIB", rubypath)
        .arg(script_path)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .context("Failed to spawn `ruby` when running script")?
        .wait()
        .context("Failed to wait for `ruby` when running script")?;
    if !status.success() {
        bail!("running `ruby` failed");
    }
    Ok(())
}

fn generate_sources(out_dir: &Path, test_helper: &UniFFITestHelper) -> Result<()> {
    for source in test_helper.get_compile_sources()? {
        let mut cmd_line = vec!["uniffi-bindgen-ruby".to_string(), "--out-dir".to_string(), out_dir.to_string_lossy().to_string(), "--no-format".to_string()];
        if let Some(path) = source.config_path {
            cmd_line.push("--config-path".to_string());
            cmd_line.push(path.to_string_lossy().to_string())
        }
        cmd_line.push(source.udl_path.to_string_lossy().to_string());
        uniffi_bindgen_ruby::run(cmd_line)?;
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
    (test_arithmetic, "uniffi-example-arithmetic", "scripts/test_arithmetic.rb"),
    (test_geometry,   "uniffi-example-geometry",   "scripts/test_geometry.rb"),
    (test_rondpoint,  "uniffi-example-rondpoint",  "scripts/test_rondpoint.rb"),
    (test_sprites,    "uniffi-example-sprites",    "scripts/test_sprites.rb"),
    (test_todolist,   "uniffi-example-todolist",   "scripts/test_todolist.rb"),
    // Fixtures
    (test_coverall,       "uniffi-fixture-coverall",       "scripts/test_coverall.rb"),
    (test_external_types, "uniffi-fixture-external-types", "scripts/test_external_types.rb"),
}
