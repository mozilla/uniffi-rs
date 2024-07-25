uniffi::build_foreign_language_testcases!(
    "tests/bindings/test_docstring.kts",
    "tests/bindings/test_docstring.swift",
    "tests/bindings/test_docstring.py",
);

#[cfg(test)]
mod tests {
    use uniffi_bindgen::{bindings::*, BindingGenerator};
    use uniffi_testing::UniFFITestHelper;

    const DOCSTRINGS: &[&str] = &[
        "<docstring-alternate-constructor>",
        "<docstring-associated-enum-variant-2>",
        "<docstring-associated-enum-variant>",
        "<docstring-associated-enum>",
        "<docstring-associated-error-variant-2>",
        "<docstring-associated-error-variant>",
        "<docstring-associated-error>",
        "<docstring-callback-method>",
        "<docstring-callback>",
        "<docstring-enum-variant-2>",
        "<docstring-enum-variant>",
        "<docstring-enum>",
        "<docstring-error-variant-2>",
        "<docstring-error-variant>",
        "<docstring-error>",
        "<docstring-function>",
        "<docstring-method>",
        "<docstring-namespace>",
        "<docstring-object>",
        "<docstring-primary-constructor>",
        "<docstring-record-field>",
        "<docstring-record>",
        "<docstring-variant-field>",
    ];

    fn test_docstring<T: BindingGenerator>(gen: T, file_extension: &str) {
        let test_helper = UniFFITestHelper::new(std::env!("CARGO_PKG_NAME")).unwrap();

        let out_dir = test_helper
            .create_out_dir(
                std::env!("CARGO_TARGET_TMPDIR"),
                format!(
                    "test-docstring-proc-macro-{}",
                    file_extension.to_string().replace('.', "")
                ),
            )
            .unwrap();

        let cdylib_path = test_helper.copy_cdylib_to_out_dir(&out_dir).unwrap();

        uniffi_bindgen::library_mode::generate_bindings(
            &cdylib_path,
            None,
            &gen,
            None,
            &out_dir,
            false,
            true,
        )
        .unwrap();

        let glob_pattern = out_dir.join(format!("**/*.{}", file_extension));

        let sources = glob::glob(glob_pattern.as_str())
            .unwrap()
            .flatten()
            .map(|p| String::from(p.to_string_lossy()))
            .collect::<Vec<String>>();

        assert_eq!(sources.len(), 1);

        let bindings_source = std::fs::read_to_string(&sources[0]).unwrap();

        let expected: Vec<String> = vec![];
        assert_eq!(
            expected,
            DOCSTRINGS
                .iter()
                .filter(|v| !bindings_source.contains(*v))
                .map(|v| v.to_string())
                .collect::<Vec::<_>>(),
            "docstrings not found in {}",
            &sources[0]
        );
    }

    #[test]
    fn test_docstring_kotlin() {
        test_docstring(KotlinBindingGenerator, "kt");
    }

    #[test]
    fn test_docstring_python() {
        test_docstring(PythonBindingGenerator, "py");
    }

    #[test]
    fn test_docstring_swift() {
        test_docstring(SwiftBindingGenerator, "swift");
    }
}
