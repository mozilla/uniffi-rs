use super::render_test_script;
use bindings_ir::ir::Module;
use camino::Utf8Path;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tera::Result;

pub fn render_test_suite(module: Module, path: &Utf8Path) -> Result<()> {
    let source = render_test_script(module)?;
    Ok(write!(File::create(path)?, "{}", source)?)
}

pub fn run_test_suite(test_dir: &Utf8Path, script_file: &Utf8Path) {
    let mut cmd = Command::new("kotlinc");
    // Start with the system classpath, we need that to find JNA
    // Add the test dir so that JNA can load the library
    cmd.arg("-classpath").arg(format!(
        "{}:{test_dir}",
        env::var_os("CLASSPATH")
            .unwrap_or_default()
            .into_string()
            .expect("Error reading CLASSPATH env")
    ));
    // Enable runtime assertions, for easy testing etc.
    cmd.arg("-J-ea");
    // Supress warnings from the generated code
    cmd.arg("-nowarn");
    cmd.arg("-script").arg(script_file);
    let status = cmd
        .spawn()
        .expect("Failed to spawn `kotlinc` to run Kotlin script")
        .wait()
        .expect("Failed to wait for `kotlinc` when running Kotlin script");
    if !status.success() {
        panic!("running `kotlinc` failed")
    }
}

fn run_test(test_dir: &Utf8Path, suite: Module) {
    let script_path = test_dir.join("test_script.kts");
    println!("Running {script_path}");
    render_test_suite(suite, &script_path).unwrap();
    run_test_suite(test_dir, &script_path);
}

#[test]
fn kotlin_basic() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-basic"),
        bindings_ir_tests::basic_tests(),
    );
}

#[test]
fn kotlin_math() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-math"),
        bindings_ir_tests::math_tests(),
    );
}

#[test]
fn kotlin_ffi_calls() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-ffi-calls"),
        bindings_ir_tests::ffi_tests(),
    );
}

#[test]
fn kotlin_compounds() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-compounds"),
        bindings_ir_tests::compound_tests(),
    );
}

#[test]
fn kotlin_pointers() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-pointers"),
        bindings_ir_tests::pointer_tests(),
    );
}

#[test]
fn kotlin_cstructs() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-cstructs"),
        bindings_ir_tests::cstructs_tests(),
    );
}

#[test]
fn kotlin_strings() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-strings"),
        bindings_ir_tests::string_tests(),
    );
}

#[test]
fn kotlin_classes() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-classes"),
        bindings_ir_tests::class_tests(),
    );
}

#[test]
fn kotlin_enums() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-enums"),
        bindings_ir_tests::enum_tests(),
    );
}

#[test]
fn kotlin_exceptions() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-exceptions"),
        bindings_ir_tests::exception_tests(),
    );
}

#[test]
fn kotlin_control_flow() {
    run_test(
        &bindings_ir_tests::setup_test_dir("kotlin-control-flow"),
        bindings_ir_tests::control_flow_tests(),
    );
}
