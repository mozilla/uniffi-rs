/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate uniffi_bindgen_tests;

#[cfg(test)]
mod test {
    use std::{
        env::{self},
        ffi::OsString,
        fs,
        process::Command,
        sync::OnceLock,
    };

    use camino::{Utf8Path, Utf8PathBuf};

    use uniffi::TargetLanguage;
    use uniffi_bindgen_tests::test_util;

    #[test]
    fn test_simple_fns() {
        run_tests(test_dir(), "tests/simple_fns.rb");
    }

    #[test]
    fn test_primitive_types() {
        run_tests(test_dir(), "tests/primitive_types.rb");
    }

    #[test]
    fn test_records() {
        run_tests(test_dir(), "tests/records.rb");
    }

    #[test]
    fn test_enums() {
        run_tests(test_dir(), "tests/enums.rb");
    }

    #[test]
    fn test_compound_types() {
        run_tests(test_dir(), "tests/compound_types.rb");
    }

    #[test]
    fn test_interfaces() {
        run_tests(test_dir(), "tests/interfaces.rb");
    }

    #[test]
    fn test_errors() {
        run_tests(test_dir(), "tests/errors.rb");
    }

    #[test]
    fn test_defaults() {
        run_tests(test_dir(), "tests/defaults.rb");
    }

    #[test]
    fn test_references() {
        run_tests(test_dir(), "tests/references.rb");
    }

    #[test]
    fn test_custom_types() {
        run_tests(test_dir(), "tests/custom_types.rb");
    }

    #[test]
    fn test_bytes() {
        run_tests(test_dir(), "tests/bytes.rb");
    }

    #[test]
    fn test_recursive_types() {
        run_tests(test_dir(), "tests/recursive_types.rb");
    }

    #[test]
    fn test_renames() {
        run_tests(test_dir(), "tests/renames.rb");
    }

    #[test]
    fn test_time() {
        run_tests(test_dir(), "tests/time.rb");
    }

    #[test]
    fn test_rust_traits() {
        run_tests(test_dir(), "tests/rust_traits.rb");
    }

    fn test_dir() -> &'static Utf8Path {
        static TEST_TEMPDIR: OnceLock<Utf8PathBuf> = OnceLock::new();
        TEST_TEMPDIR.get_or_init(|| {
            let temp_dir = test_util::setup_test_dir("ruby");
            let lib_dir = temp_dir.join("lib");
            fs::create_dir_all(&lib_dir).unwrap();
            test_util::build_library(&lib_dir);
            test_util::generate_sources(&lib_dir, TargetLanguage::Ruby);
            test_util::copy_test_sources(&temp_dir, "tests/*.rb");
            temp_dir
        })
    }

    fn run_tests(tempdir: &Utf8Path, script_filename: &str) {
        let lib_dir = tempdir.join("lib");
        let script_path = tempdir.join(script_filename);
        let rubypath = env::var_os("RUBYLIB").unwrap_or_else(|| OsString::from(""));
        let rubypath = env::join_paths(
            env::split_paths(&rubypath).chain(vec![lib_dir.to_path_buf().into_std_path_buf()]),
        )
        .unwrap();

        let mut command = Command::new("ruby");
        command
            // Run from lib_dir so FFI finds the dylib in the current directory
            .current_dir(lib_dir)
            .env("RUBYLIB", rubypath)
            .arg(script_path);

        let output = command
            .output()
            .expect("Failed to spawn `ruby` when running test script");
        print!("{}", String::from_utf8_lossy(&output.stdout));

        if !output.status.success() {
            println!("---------------------------------------- STDERR ----------------------------------------");
            print!("{}", String::from_utf8_lossy(&output.stderr));
            println!("----------------------------------------------------------------------------------------");
            panic!("running `ruby` to run test script failed ({:?})", command);
        }
    }
}
