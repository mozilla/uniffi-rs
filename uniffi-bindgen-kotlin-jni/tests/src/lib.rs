/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

include!(concat!(
    env!("OUT_DIR"),
    "/uniffi_bindgen_kotlin_jni.uniffi.rs"
));

#[cfg(test)]
mod test {
    use std::{env, process::Command, sync::OnceLock};

    use camino::{Utf8Path, Utf8PathBuf};

    use uniffi_bindgen_tests::test_util;

    #[test]
    fn test_simple_fns() {
        run_tests(test_dir(), "tests/simple_fns.kts");
    }

    #[test]
    fn test_primitive_types() {
        run_tests(test_dir(), "tests/primitive_types.kts");
    }

    #[test]
    fn test_records() {
        run_tests(test_dir(), "tests/records.kts");
    }

    #[test]
    fn test_enums() {
        run_tests(test_dir(), "tests/enums.kts");
    }

    #[test]
    fn test_compound_types() {
        run_tests(test_dir(), "tests/compound_types.kts");
    }

    #[test]
    fn test_interfaces() {
        run_tests(test_dir(), "tests/interfaces.kts");
    }

    #[test]
    fn test_custom_types() {
        run_tests(test_dir(), "tests/custom_types.kts");
    }

    #[test]
    fn test_errors() {
        run_tests(test_dir(), "tests/errors.kts");
    }

    #[test]
    fn test_callback_interfaces() {
        run_tests(test_dir(), "tests/callback_interfaces.kts");
    }

    #[test]
    fn test_futures() {
        run_tests(test_dir(), "tests/futures.kts");
    }

    #[test]
    fn test_trait_interfaces() {
        run_tests(test_dir(), "tests/trait_interfaces.kts");
    }

    #[test]
    fn test_defaults() {
        run_tests(test_dir(), "tests/defaults.kts");
    }

    #[test]
    fn test_external_types() {
        run_tests(test_dir(), "tests/external_types.kts");
    }

    #[test]
    fn test_renames() {
        run_tests(test_dir(), "tests/renames.kts");
    }

    #[test]
    fn test_recursive_types() {
        run_tests(test_dir(), "tests/recursive_types.kts");
    }

    fn test_dir() -> &'static Utf8Path {
        static TEST_TEMPDIR: OnceLock<Utf8PathBuf> = OnceLock::new();
        TEST_TEMPDIR.get_or_init(|| {
            let temp_dir = test_util::setup_test_dir("kotlin");
            test_util::build_library(&temp_dir);
            test_util::copy_test_sources(&temp_dir, "tests/*.kts");
            uniffi_bindgen_kotlin_jni::generate_bindings(
                "src:uniffi_bindgen_kotlin_jni_tests",
                &temp_dir,
            )
            .unwrap();
            build_jar(&temp_dir);
            temp_dir
        })
    }

    fn run_tests(tempdir: &Utf8Path, script_filename: &str) {
        run_test_script(tempdir, script_filename);
    }

    fn build_jar(temp_dir: &Utf8Path) -> Utf8PathBuf {
        let jar_file = temp_dir.join("uniffi_bindgen_tests.jar");
        let glob_spec = temp_dir.join("uniffi/**/*.kt");
        let sources = glob::glob(glob_spec.as_str())
            .unwrap()
            .flatten()
            .map(|p| String::from(p.to_string_lossy()))
            .collect::<Vec<String>>();
        if sources.is_empty() {
            panic!("No sources found ({glob_spec})");
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

    fn run_test_script(temp_dir: &Utf8Path, script_file: &str) {
        let jar_file = temp_dir.join("uniffi_bindgen_tests.jar");
        let mut command = Command::new("kotlinc");
        command
            .arg("-classpath")
            .arg(calc_classpath(vec![temp_dir, &jar_file]))
            // Enable runtime assertions, for easy testing etc.
            .arg("-J-ea")
            // Our test scripts should not produce any warnings.
            .arg("-Werror")
            .arg("-script")
            .arg(script_file);

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
}
