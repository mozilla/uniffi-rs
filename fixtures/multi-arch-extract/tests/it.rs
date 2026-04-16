// Needs macOS so we can compile for both x86_64-apple-darwin and aarch64-apple-darwin
#![cfg(target_os = "macos")]

use std::fs;

use tempfile::{Builder, NamedTempFile};
use xshell::{cmd, Shell};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

// The most minimal code equivalent to what UniFFI generates:
// A single custom type for a crate named `fixture` with a type name of `custom`.
static C_CODE: &str = r#"
unsigned char UNIFFI_META_FIXTURE_CUSTOM_TYPE_CUSTOM[18] = {
  15,
  7, 'f', 'i', 'x', 't', 'u', 'r', 'e',
  6, 'c', 'u', 's', 't', 'o', 'm',
  0, 0
};
"#;

fn tempfile(prefix: &str, suffix: &str) -> Result<NamedTempFile, std::io::Error> {
    Builder::new().prefix(prefix).suffix(suffix).tempfile()
}

#[test]
fn extraction_from_multi_arch_lib_works() -> Result<()> {
    let code_file = tempfile("custom", ".c")?;
    let code_file_path = code_file.path();
    fs::write(code_file_path, C_CODE)?;

    let x64 = tempfile("x64", ".dylib")?;
    let x64_path = x64.path();
    let arm64 = tempfile("arm64", ".dylib")?;
    let arm64_path = arm64.path();
    let multiarch = tempfile("multiarch", ".dylib")?;
    let multiarch_path = multiarch.path();

    // We build 2 dylibs (for x86_64 and arm64),
    // then combine them into a multi-arch dylib (using `lipo`)

    let sh = Shell::new()?;
    cmd!(
        sh,
        "cc -shared -O3 -target x86_64-apple-darwin -o {x64_path} {code_file_path}"
    )
    .run()?;
    cmd!(
        sh,
        "cc -shared -O3 -target aarch64-apple-darwin -o {arm64_path} {code_file_path}"
    )
    .run()?;

    cmd!(
        sh,
        "lipo -create -output {multiarch_path} {x64_path} {arm64_path}"
    )
    .run()?;

    let dylib_bytes = fs::read(multiarch_path)?;

    let metadata = uniffi_bindgen::macro_metadata::extract_from_bytes(&dylib_bytes)?;
    assert_eq!(1, metadata.len());

    Ok(())
}
