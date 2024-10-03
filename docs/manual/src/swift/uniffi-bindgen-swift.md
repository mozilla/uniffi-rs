# uniffi-bindgen-swift

Swift bindings can be generated like other languages using `uniffi-bindgen -l swift`.  However, you
can also use the `uniffi-bindgen-swift` binary which gives greater control over Swift-specific
features:

* Select which kind of files to generate: headers, modulemaps, and/or Swift sources.
* Generate a single modulemap for a library.
* Generate XCFramework-compatible modulemaps.
* Customize the modulemap module name.
* Customize the modulemap filename.

`uniffi-bindgen-swift` can be added to your project using the same general steps as `uniffi-bindgen`.
See https://mozilla.github.io/uniffi-rs/latest/tutorial/foreign_language_bindings.html#creating-the-bindgen-binary.
The Rust source for the binary should be:

```
fn main() {
    uniffi::uniffi_bindgen_swift()
}
```

`uniffi-bindgen-swift` always inputs a library path and runs in "library mode".  This means
proc-macro-based bindings generation is always supported.

## Examples:


Generate .swift source files for a library
```
cargo run -p uniffi-bindgen-swift -- target/release/mylibrary.a build/swift --swift-sources
```

Generate .h files for a library
```
cargo run -p uniffi-bindgen-swift -- target/release/mylibrary.a build/swift/Headers --headers
```


Generate a modulemap
```
cargo run -p uniffi-bindgen-swift -- target/release/mylibrary.a build/swift/Modules --modulemap --modulemap-filename mymodule.modulemap
```

Generate a Xcframework-compatible modulemap
```
cargo run -p uniffi-bindgen-swift -- target/release/mylibrary.a build/swift/Modules --xcframework --modulemap --modulemap-filename mymodule.modulemap
```
