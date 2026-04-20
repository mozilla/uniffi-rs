/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

include!(concat!(
    env!("OUT_DIR"),
    "/uniffi_bindgen_kotlin_jni.uniffi.rs"
));

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use camino::{Utf8Path, Utf8PathBuf};

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
    fn test_bytes() {
        run_tests(test_dir(), "tests/bytes.kts");
    }

    #[test]
    fn test_recursive_types() {
        run_tests(test_dir(), "tests/recursive_types.kts");
    }

    #[test]
    fn test_time() {
        run_tests(test_dir(), "tests/time.kts");
    }

    #[test]
    fn test_rust_traits() {
        run_tests(test_dir(), "tests/rust_traits.kts");
    }

    fn test_dir() -> &'static Utf8Path {
        static TEST_TEMPDIR: OnceLock<Utf8PathBuf> = OnceLock::new();
        TEST_TEMPDIR.get_or_init(|| {
            let temp_dir = uniffi_bindgen::test_util::setup_test_dir("kotlin");
            uniffi_bindgen::test_util::build_library(
                &temp_dir,
                "uniffi-bindgen-kotlin-jni-tests",
                uniffi_bindgen::test_util::LibraryOptions::default(),
            );
            uniffi_bindgen::test_util::copy_test_sources(&temp_dir, "tests/*.kts");
            uniffi_bindgen_kotlin_jni::test_util::generate_bindings(&temp_dir);
            uniffi_bindgen_kotlin_jni::test_util::build_jar(
                &temp_dir,
                "uniffi_bindgen_tests.jar",
                "uniffi/**/*.kt",
            );
            temp_dir
        })
    }

    fn run_tests(tempdir: &Utf8Path, script_filename: &str) {
        uniffi_bindgen_kotlin_jni::test_util::run_script(
            tempdir,
            "uniffi_bindgen_tests.jar",
            script_filename,
            uniffi_bindgen_kotlin_jni::test_util::RunScriptOptions {
                capture_output: true,
                ..Default::default()
            },
        );
    }
}
