// uniffi_macros::build_foreign_language_testcases!(
//     "src/imported-types-lib.udl",
//     [
//         "tests/bindings/test_imported_types.py",
//     ]
// );

use std::env;
use std::path::PathBuf;

// Ideally we'd still use a macro to build language test cases, but markh
// failed with the macro magic needed to make multiple udl files work.
#[test]
fn test_exttypes_python() -> uniffi::deps::anyhow::Result<()> {
    let pkg_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Missing $CARGO_MANIFEST_DIR, cannot build tests for generated bindings");

    let udl_files = [
        "../guid/src/guid.udl",
        "../../../examples/wrapper-types/src/wrapper-types.udl",
        "../uniffi-one/src/uniffi-one.udl",
        "src/ext-types-lib.udl",
    ];
    let file_path = "tests/bindings/test_imported_types.py";
    let test_file_pathbuf: PathBuf = [&pkg_dir, file_path].iter().collect();
    let test_file_path = test_file_pathbuf.to_string_lossy();

    uniffi::testing::run_foreign_language_testcase(&pkg_dir, &udl_files, &test_file_path)
}

#[test]
fn test_exttypes_kt() -> uniffi::deps::anyhow::Result<()> {
    let pkg_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Missing $CARGO_MANIFEST_DIR, cannot build tests for generated bindings");

    let udl_files = [
        "../guid/src/guid.udl",
        "../../../examples/wrapper-types/src/wrapper-types.udl",
        "../uniffi-one/src/uniffi-one.udl",
        "src/ext-types-lib.udl",
    ];
    let file_path = "tests/bindings/test_imported_types.kts";
    let test_file_pathbuf: PathBuf = [&pkg_dir, file_path].iter().collect();
    let test_file_path = test_file_pathbuf.to_string_lossy();

    uniffi::testing::run_foreign_language_testcase(&pkg_dir, &udl_files, &test_file_path)
}
