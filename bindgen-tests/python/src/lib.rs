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
        io::Write,
        process::Command,
        sync::OnceLock,
    };

    use camino::{Utf8Path, Utf8PathBuf};

    use uniffi::TargetLanguage;
    use uniffi_bindgen_tests::test_util;

    #[test]
    fn test_simple_fns() {
        run_tests(test_dir(), "tests/simple_fns.py");
    }

    #[test]
    fn test_primitive_types() {
        run_tests(test_dir(), "tests/primitive_types.py");
    }

    #[test]
    fn test_records() {
        run_tests(test_dir(), "tests/records.py");
    }

    #[test]
    fn test_compound_types() {
        run_tests(test_dir(), "tests/compound_types.py");
    }

    #[test]
    fn test_enums() {
        run_tests(test_dir(), "tests/enums.py");
    }

    #[test]
    fn test_interfaces() {
        run_tests(test_dir(), "tests/interfaces.py");
    }

    #[test]
    fn test_errors() {
        run_tests(test_dir(), "tests/errors.py");
    }

    #[test]
    fn test_callback_interfaces() {
        run_tests(test_dir(), "tests/callback_interfaces.py");
    }

    #[test]
    fn test_futures() {
        run_tests(test_dir(), "tests/futures.py");
    }

    #[test]
    fn test_trait_interfaces() {
        run_tests(test_dir(), "tests/trait_interfaces.py");
    }

    #[test]
    fn test_complex_fns() {
        run_tests(test_dir(), "tests/complex_fns.py");
    }

    #[test]
    fn test_custom_types() {
        run_tests(test_dir(), "tests/custom_types.py");
    }

    #[test]
    fn test_external_types() {
        run_tests(test_dir(), "tests/external_types.py");
    }

    #[test]
    fn test_renames() {
        run_tests(test_dir(), "tests/renames.py");
    }

    fn test_dir() -> &'static Utf8Path {
        static TEST_TEMPDIR: OnceLock<Utf8PathBuf> = OnceLock::new();
        TEST_TEMPDIR.get_or_init(|| {
            let temp_dir = test_util::setup_test_dir("python");
            let test_package = temp_dir.join("test_package");
            fs::create_dir(&test_package).unwrap();
            let mut f = fs::File::create(test_package.join("__init__.py")).unwrap();
            write!(f, "").unwrap();
            test_util::build_library(&test_package);
            test_util::copy_test_sources(&temp_dir, "tests/*.py");
            test_util::generate_sources(&test_package, TargetLanguage::Python);

            temp_dir
        })
    }

    fn run_tests(tempdir: &Utf8Path, script_filename: &str) {
        // Run the test script against compiled bindings
        let pythonpath = env::var_os("PYTHONPATH").unwrap_or_else(|| OsString::from(""));
        let pythonpath = env::join_paths(
            env::split_paths(&pythonpath).chain(vec![tempdir.to_path_buf().into_std_path_buf()]),
        )
        .unwrap();
        let mut command = Command::new("python3");
        command
            .current_dir(tempdir)
            .env("PYTHONPATH", pythonpath)
            .arg(script_filename);
        let output = command
            .output()
            .expect("Failed to spawn `python3` when running test script");
        // The `output()` call above sets up pipes to print stdout/stderr.  This allows it to be
        // integrated with the Rust test harness's output handling.
        print!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            println!("---------------------------------------- STDERR ----------------------------------------");
            print!("{}", String::from_utf8_lossy(&output.stderr));
            println!("----------------------------------------------------------------------------------------");
            panic!("running `python` to run test script failed ({:?})", command);
        }
    }
}
