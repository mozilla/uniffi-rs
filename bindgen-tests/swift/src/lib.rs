/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(test)]
mod test {
    use std::{
        env::consts::{DLL_PREFIX, DLL_SUFFIX},
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
        run_tests(test_dir(), "tests/simple_fns.swift");
    }

    #[test]
    fn test_primitive_types() {
        run_tests(test_dir(), "tests/primitive_types.swift");
    }

    #[test]
    fn test_records() {
        run_tests(test_dir(), "tests/records.swift");
    }

    #[test]
    fn test_compound_types() {
        run_tests(test_dir(), "tests/compound_types.swift");
    }

    #[test]
    fn test_enums() {
        run_tests(test_dir(), "tests/enums.swift");
    }

    #[test]
    fn test_interfaces() {
        run_tests(test_dir(), "tests/interfaces.swift");
    }

    #[test]
    fn test_errors() {
        run_tests(test_dir(), "tests/errors.swift");
    }

    #[test]
    fn test_callback_interfaces() {
        run_tests(test_dir(), "tests/callback_interfaces.swift");
    }

    #[test]
    fn test_futures() {
        run_tests(test_dir(), "tests/futures.swift");
    }

    #[test]
    fn test_trait_interfaces() {
        run_tests(test_dir(), "tests/trait_interfaces.swift");
    }

    #[test]
    fn test_complex_fns() {
        run_tests(test_dir(), "tests/complex_fns.swift");
    }

    #[test]
    fn test_custom_types() {
        run_tests(test_dir(), "tests/custom_types.swift");
    }

    #[test]
    fn test_external_types() {
        run_tests(test_dir(), "tests/external_types.swift");
    }

    #[test]
    fn test_renames() {
        run_tests(test_dir(), "tests/renames.swift");
    }

    fn test_dir() -> &'static Utf8Path {
        static TEST_TEMPDIR: OnceLock<Utf8PathBuf> = OnceLock::new();
        TEST_TEMPDIR.get_or_init(|| {
            let temp_dir = test_util::setup_test_dir("swift");
            test_util::build_library(&temp_dir);
            test_util::generate_sources(&temp_dir, TargetLanguage::Swift);
            test_util::copy_test_sources(&temp_dir, "tests/*.swift");
            combine_module_maps(&temp_dir);
            compile_sources(&temp_dir);
            temp_dir
        })
    }

    // We need something better than this env var, but it's a reasonable start.
    fn swift_version() -> String {
        std::env::var("UNIFFI_TEST_SWIFT_VERSION").unwrap_or("5".to_string())
    }

    // uniffi-bindgen creates a separate modulemap for each Rust crate.
    //
    // Combine all of these into a single modulemap so that the tests will run
    fn combine_module_maps(tempdir: &Utf8Path) {
        let mut f = fs::File::create(tempdir.join("combined.modulemap")).unwrap();
        let source_filenames = [
            "uniffi_bindgen_testsFFI.modulemap",
            "uniffi_bindgen_tests_external_types_sourceFFI.modulemap",
        ];
        for filename in source_filenames {
            let path = tempdir.join(filename);
            let contents = fs::read_to_string(path).unwrap();
            write!(f, "{contents}").unwrap();
        }
    }

    fn compile_sources(tempdir: &Utf8Path) {
        let mut command = Command::new("swiftc");
        command
            .current_dir(tempdir)
            .arg("-module-name")
            .arg("uniffi_bindgen_tests")
            .arg("-emit-module")
            .arg("-parse-as-library")
            // TODO(2279): Fix concurrency issues and uncomment this
            //.arg("-strict-concurrency=complete")
            .arg("-o")
            .arg(format!("{DLL_PREFIX}test_library{DLL_SUFFIX}"))
            .arg("-emit-library")
            .arg("-swift-version")
            .arg(swift_version())
            .arg("-Xcc")
            .arg(format!("-fmodule-map-file={tempdir}/combined.modulemap"))
            .arg("-L")
            .arg(tempdir)
            .arg("uniffi_bindgen_tests.swift")
            .arg("uniffi_bindgen_tests_external_types_source.swift");
        let status = command
            .spawn()
            .expect("Failed to spawn `swiftc` when compiling bindings")
            .wait()
            .expect("Failed to wait for `swiftc` when compiling bindings");
        if !status.success() {
            panic!(
                "running `swiftc` to compile bindings failed ({:?})",
                command
            )
        };
    }

    fn run_tests(tempdir: &Utf8Path, script_filename: &str) {
        // Run the test script against compiled bindings
        let mut command = Command::new("swift");
        let output = command
            .current_dir(tempdir)
            .env("SWIFT_BACKTRACE", "interactive=no")
            .arg("-I")
            .arg(tempdir)
            .arg("-L")
            .arg(tempdir)
            .arg("-ltest_library")
            .arg("-luniffi_bindgen_tests")
            .arg("-swift-version")
            .arg(swift_version())
            .arg("-Xcc")
            .arg(format!("-fmodule-map-file={tempdir}/combined.modulemap"))
            .arg(script_filename)
            .output()
            .expect("Failed to spawn `swiftc` when running test script");
        // The `output()` call above sets up pipes to print stdout/stderr.  This allows it to be
        // integrated with the Rust test harness's output handling.
        if !output.stderr.is_empty() {
            println!("---------------------------------------- STDERR ----------------------------------------");
            print!("{}", String::from_utf8_lossy(&output.stderr));
            println!("----------------------------------------------------------------------------------------");
            panic!("swift outputed standard error");
        }
        print!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            panic!("running `swift` to run test script failed ({:?})", command)
        }
    }
}
