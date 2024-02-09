use camino::Utf8PathBuf;
use std::process::Command;
use uniffi::SwiftBindingGenerator;
use uniffi_testing::UniFFITestHelper;

#[test]
fn clang() -> Result<(), anyhow::Error> {
    let tmp_dir = std::env!("CARGO_TARGET_TMPDIR");
    let crate_name = std::env!("CARGO_PKG_NAME");

    let test_helper = UniFFITestHelper::new(crate_name)?;
    let out_dir = test_helper.create_out_dir(tmp_dir, "clang.rs")?;

    uniffi::generate_bindings(
        &Utf8PathBuf::from("src/swift-bridging-header-compile.udl"),
        None,
        SwiftBindingGenerator,
        Some(&out_dir),
        None,
        None,
        false,
    )?;

    let bridging_h = out_dir.join("swift_bridging_header_compileFFI.h");

    // Compile the header as objective-c with a pedantic set of warnings.
    let o = Command::new("clang")
        .args([
            "-c",
            "-x",
            "objective-c",
            "-Wpedantic",
            "-Werror",
            "-Wstrict-prototypes",
            "-Wno-pragma-once-outside-header", // We are compiling a header directly, so this is fine.
            "-Wno-newline-eof",                // If `swiftformat` were used this would be ok.
            "-o",
            "/dev/null",
            bridging_h.as_str(),
        ])
        .output()?;

    assert!(
        o.status.success(),
        r#"Failed to compile bridging header {}:
stdout:
{}

stderr:
{}
"#,
        o.status,
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    Ok(())
}
