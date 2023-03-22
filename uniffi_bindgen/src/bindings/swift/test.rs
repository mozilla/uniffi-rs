/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use heck::ToSnakeCase;
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::ffi::OsStr;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::Command;
use uniffi_testing::{CompileSource, UniFFITestHelper};

/// Run Swift tests for a UniFFI test fixture
pub fn run_test(tmp_dir: &str, fixture_name: &str, script_file: &str) -> Result<()> {
    let script_path = Utf8Path::new(".").join(script_file).canonicalize_utf8()?;
    let test_helper = UniFFITestHelper::new(fixture_name).context("UniFFITestHelper::new")?;
    let out_dir = test_helper
        .create_out_dir(tmp_dir, &script_path)
        .context("create_out_dir()")?;
    test_helper
        .copy_cdylibs_to_out_dir(&out_dir)
        .context("copy_fixture_library_to_out_dir()")?;
    let generated_sources =
        GeneratedSources::new(&test_helper.cdylib_path()?, &out_dir, &test_helper)
            .context("generate_sources()")?;

    // Compile the generated sources together to create a single swift module
    compile_swift_module(
        &out_dir,
        &calc_module_name(&generated_sources.main_source_filename),
        &generated_sources.generated_swift_files,
        &generated_sources.module_map,
    )?;

    // Run the test script against compiled bindings

    let mut command = Command::new("swift");
    command
        .current_dir(&out_dir)
        .arg("-I")
        .arg(&out_dir)
        .arg("-L")
        .arg(&out_dir)
        .args(calc_library_args(&out_dir)?)
        .arg("-Xcc")
        .arg(format!(
            "-fmodule-map-file={}",
            generated_sources.module_map
        ))
        .arg(&script_path);
    let status = command
        .spawn()
        .context("Failed to spawn `swiftc` when running test script")?
        .wait()
        .context("Failed to wait for `swiftc` when running test script")?;
    if !status.success() {
        bail!("running `swift` to run test script failed ({:?})", command)
    }
    Ok(())
}

fn calc_module_name(filename: &str) -> String {
    filename.strip_suffix(".swift").unwrap().to_snake_case()
}

fn compile_swift_module<T: AsRef<OsStr>>(
    out_dir: &Utf8Path,
    module_name: &str,
    sources: impl IntoIterator<Item = T>,
    module_map: &Utf8Path,
) -> Result<()> {
    let output_filename = format!("{DLL_PREFIX}testmod_{module_name}{DLL_SUFFIX}");
    let mut command = Command::new("swiftc");
    command
        .current_dir(out_dir)
        .arg("-emit-module")
        .arg("-module-name")
        .arg(module_name)
        .arg("-o")
        .arg(output_filename)
        .arg("-emit-library")
        .arg("-Xcc")
        .arg(format!("-fmodule-map-file={module_map}"))
        .arg("-I")
        .arg(out_dir)
        .arg("-L")
        .arg(out_dir)
        .args(calc_library_args(out_dir)?)
        .args(sources);
    let status = command
        .spawn()
        .context("Failed to spawn `swiftc` when compiling bindings")?
        .wait()
        .context("Failed to wait for `swiftc` when compiling bindings")?;
    if !status.success() {
        bail!(
            "running `swiftc` to compile bindings failed ({:?})",
            command
        )
    };
    Ok(())
}

// Stores sources generated by `uniffi-bindgen-swift`
struct GeneratedSources {
    generated_swift_files: Vec<Utf8PathBuf>,
    module_map: Utf8PathBuf,
    main_source_filename: String,
}

impl GeneratedSources {
    fn new(
        library_path: &Utf8Path,
        out_dir: &Utf8Path,
        test_helper: &UniFFITestHelper,
    ) -> Result<Self> {
        // Generate the bindings for the main compile source, and use that for the swift module name
        let main_compile_source = test_helper.get_main_compile_source()?;
        Self::run_generate_bindings(&main_compile_source, library_path, out_dir)?;
        let generated_files = glob(&out_dir.join("*.swift"))?;
        let main_source_filename = match generated_files.len() {
            0 => bail!(
                "No .swift file generated for {}",
                main_compile_source.udl_path
            ),
            1 => generated_files
                .into_iter()
                .next()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string(),
            n => bail!(
                "{n} .swift files generated for {}",
                main_compile_source.udl_path
            ),
        };

        // Generate the bindings for other compile sources (crates used by external types)
        for source in test_helper.get_external_compile_sources()? {
            crate::generate_bindings(
                &source.udl_path,
                source.config_path.as_deref(),
                vec!["swift"],
                Some(out_dir),
                Some(library_path),
                false,
            )?;
        }

        let generated_module_maps = glob(&out_dir.join("*.modulemap"))?;

        Ok(GeneratedSources {
            main_source_filename,
            generated_swift_files: glob(&out_dir.join("*.swift"))?,
            module_map: match generated_module_maps.len() {
                0 => bail!("No modulemap files found in {out_dir}"),
                // Normally we only generate 1 module map and can return it directly
                1 => generated_module_maps.into_iter().next().unwrap(),
                // When we use multiple UDL files in a test, for example the ext-types fixture,
                // then we get multiple module maps and need to combine them
                _ => {
                    let path = out_dir.join("combined.modulemap");
                    let mut f = File::create(&path)?;
                    write!(
                        f,
                        "{}",
                        generated_module_maps
                            .into_iter()
                            .map(|path| Ok(read_to_string(path)?))
                            .collect::<Result<Vec<String>>>()?
                            .join("\n")
                    )?;
                    path
                }
            },
        })
    }

    fn run_generate_bindings(
        source: &CompileSource,
        library_path: &Utf8Path,
        out_dir: &Utf8Path,
    ) -> Result<()> {
        crate::generate_bindings(
            &source.udl_path,
            source.config_path.as_deref(),
            vec!["swift"],
            Some(out_dir),
            Some(library_path),
            false,
        )
    }
}

// Wraps glob to use Utf8Paths and flattens errors
fn glob(globspec: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    glob::glob(globspec.as_str())?
        .map(|globresult| Ok(Utf8PathBuf::try_from(globresult?)?))
        .collect()
}

fn calc_library_args(out_dir: &Utf8Path) -> Result<Vec<String>> {
    let results = glob::glob(out_dir.join(format!("{DLL_PREFIX}*{DLL_SUFFIX}")).as_str())?;
    results
        .map(|globresult| {
            let path = Utf8PathBuf::try_from(globresult.unwrap())?;
            Ok(format!(
                "-l{}",
                path.file_name()
                    .unwrap()
                    .strip_prefix(DLL_PREFIX)
                    .unwrap()
                    .strip_suffix(DLL_SUFFIX)
                    .unwrap()
            ))
        })
        .collect()
}