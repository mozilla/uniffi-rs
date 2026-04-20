/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Test utilities
//!
//! These extend the functions from `uniffi_bindgen::test_util`

use std::{env, process::Command};

use camino::{Utf8Path, Utf8PathBuf};

/// Run `uniffi-bindgen-kotlin-jni` and output to the temp dir
pub fn generate_bindings(temp_dir: &Utf8Path) {
    let package_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set");
    let lib_name = package_name.replace("-", "_");

    crate::generate_bindings(&format!("src:{lib_name}"), temp_dir)
        .expect("Error generating bindings");
}

/// Build a .jar file in the tempdir from the generated sources
pub fn build_jar(temp_dir: &Utf8Path, jar_filename: &str, globspec: &str) -> Utf8PathBuf {
    let jar_file = temp_dir.join(jar_filename);
    let glob_spec = temp_dir.join(globspec);
    let mut sources = glob::glob(glob_spec.as_str())
        .unwrap()
        .flatten()
        .map(|p| String::from(p.to_string_lossy()))
        .collect::<Vec<String>>();
    if sources.is_empty() {
        panic!("No sources found ({glob_spec})");
    }
    let uniffi_package = temp_dir.join("uniffi/Uniffi.kt").to_string();
    if !sources.contains(&uniffi_package) {
        sources.push(uniffi_package);
    }

    let mut command = Command::new("kotlinc");
    command
        // Our generated bindings should not produce any warnings; fail tests if they do.
        .arg("-Werror")
        .arg("-d")
        .arg(&jar_file)
        .arg("-classpath")
        .arg(calc_classpath(vec![]))
        .args(sources);

    let output = command
        .output()
        .expect("Failed to spawn `kotlinc` when running test script");
    // The `output()` call above sets up pipes to print stdout/stderr.  This allows it to be
    // integrated with the Rust test harness's output handling.
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        println!("---------------------------------------- STDERR ----------------------------------------");
        print!("{}", String::from_utf8_lossy(&output.stderr));
        println!("----------------------------------------------------------------------------------------");
        panic!(
            "running `kotlinc` to run test script failed ({:?})",
            command
        );
    }
    jar_file
}

/// Run a script from the temp directory
pub fn run_script(
    temp_dir: &Utf8Path,
    jar_filename: &str,
    script_file: &str,
    opts: RunScriptOptions,
) {
    let jar_file = temp_dir.join(jar_filename);
    let mut command = Command::new("kotlinc");
    let temp_dir = temp_dir.canonicalize_utf8().expect("invalid temp_dir");
    let mut extra_args = vec![];
    if let Some(gb) = opts.initial_heap_gb {
        extra_args.push(format!("-J-Xms{gb}g"))
    }
    if let Some(gb) = opts.max_heap_gb {
        extra_args.push(format!("-J-Xmx{gb}g"))
    }
    command
        .arg("-classpath")
        .arg(calc_classpath(vec![&temp_dir, &jar_file]))
        // Enable runtime assertions, for easy testing etc.
        .arg("-J-ea")
        .arg(format!("-J-Djava.library.path={temp_dir}"))
        .args(extra_args)
        // Our test scripts should not produce any warnings.
        .arg("-Werror")
        .arg("-script")
        .arg(script_file)
        .current_dir(temp_dir)
        .args(if opts.args.is_empty() {
            vec![]
        } else {
            std::iter::once(String::from("--"))
                .chain(opts.args)
                .collect()
        });

    if opts.capture_output {
        let output = command
            .output()
            .expect("Failed to spawn `kotlinc` when running test script");
        // The `output()` call above sets up pipes to print stdout/stderr.  This allows it to be
        // integrated with the Rust test harness's output handling.
        print!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            println!("---------------------------------------- STDERR ----------------------------------------");
            print!("{}", String::from_utf8_lossy(&output.stderr));
            println!("----------------------------------------------------------------------------------------");
            panic!(
                "running `kotlinc` to run test script failed ({:?})",
                command
            );
        }
    } else {
        let status = command
            .status()
            .expect("Failed to spawn `kotlinc` when running test script");
        if !status.success() {
            panic!(
                "running `kotlinc` to run test script failed ({:?})",
                command
            );
        }
    }
}

#[derive(Default)]
pub struct RunScriptOptions {
    pub args: Vec<String>,
    pub capture_output: bool,
    pub initial_heap_gb: Option<u32>,
    pub max_heap_gb: Option<u32>,
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
