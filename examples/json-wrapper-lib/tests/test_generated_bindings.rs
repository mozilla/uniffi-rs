use std::env;
use std::path::PathBuf;

// Ideally we'd still use a macro to build language test cases, but markh
// failed with the macro magic needed to make multiple udl files work.
#[test]
fn test_exttypes_python() -> uniffi::deps::anyhow::Result<()> {
    let pkg_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Missing $CARGO_MANIFEST_DIR, cannot build tests for generated bindings");

    let udl_files = [
        "../json-wrapper-ext/src/json-wrapper-ext.udl",
        "src/json-wrapper-lib.udl",
    ];
    let file_path = "tests/bindings/test_json.py";
    let test_file_pathbuf: PathBuf = [&pkg_dir, file_path].iter().collect();
    let test_file_path = test_file_pathbuf.to_string_lossy();

    uniffi::testing::run_foreign_language_testcase(&pkg_dir, &udl_files, &test_file_path)
}
