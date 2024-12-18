use uniffi::deps::anyhow::Result;

uniffi::bindings_tests!(
    py: run_python_test,
    // TODO Kotlin/Swift/Ruby
);

pub fn run_python_test(tmp_dir: &str, fixture_name: &str) -> Result<()> {
    let script_name = format!("python/{fixture_name}.py");
    uniffi::python_test::run_test(tmp_dir, fixture_name, &script_name)
}
