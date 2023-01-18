### Migrating to UniFFI 0.23

- Update your `Cargo.toml` file to only depend on the `uniffi` crate.  Follow the directions from the [Prerequisites section of the manual](https://mozilla.github.io/uniffi-rs/tutorial/Prerequisites.html)
- Create a `uniffi-bindgen` binary for your project.  Follow the directions from the [Foreign language bindings section of the manual](https://mozilla.github.io/uniffi-rs/tutorial/foreign_language_bindings.html).
- Uninstall the system-wide `uniffi_bindgen`: `cargo uninstall uniffi_bindgen`.  (Not strictly necessary, but you won't be using it anymore).

<!-- The sections in this file are managed automatically by `cargo release` -->
<!-- See our [internal release process docs](docs/release-process.md) and for more general -->
<!-- guidance, see https://github.com/sunng87/cargo-release/blob/master/docs/faq.md#maintaining-changelog -->

<!-- next-header -->

## [[UnreleasedUniFFIVersion]] (backend crates: [[UnreleasedBackendVersion]] - (_[[ReleaseDate]]_)

[All changes in [[UnreleasedVersion]]](https://github.com/mozilla/uniffi-rs/compare/v0.22.0...HEAD).

### ⚠️ Breaking Changes ⚠️

- `uniffi_bindgen` no longer provides a standalone binary.  Having a standalone binary resulted in version mismatches when the `uniffi` version specified in `Cargo.toml` didn't match the `uniffi_bindgen` version installed on the system.  Read [The foreign language bindings](https://mozilla.github.io/uniffi-rs/tutorial/foreign_language_bindings.html) section of the manual for how to set up a `uniffi-bindgen` binary that's local to your workspace.
- `uniffi_bindgen`: Removed the `run_main` function.  It's moved to `uniffi::uniffi_bindgen_main` and now unconditionally succeeds rather than return a `Result<()>`.

### What's changed

- The UniFFI crate organization has been significantly reworked:
  - Projects that use UniFFI for binding/scaffolding generation now only need to depend on the `uniffi` crate and no longer need to depend on `uniffi_bindgen`, `uniffi_build`, etc.
  - The version numbers for each crate will no longer by kept in sync after this release.  In particular `uniffi` will have breaking changes less often than `uniffi_bindgen` and other crates.  This means that UniFFI consumers that specify their versions like `uniffi = "0.23"` will not need to bump their `uniffi` version as often as before.
- Callback interface method calls are no longer logged (#1439)

## v0.22.0 - (_2022-12-16_)

[All changes in v0.22.0](https://github.com/mozilla/uniffi-rs/compare/v0.21.1...v0.22.0).

### ⚠️ Breaking Changes ⚠️

- `uniffi_bindgen`: Renamed `FFIArgument`, `FFIFunction` and `FFIType` to
  `FfiArgument`, `FfiFunction` and `FfiType`

### What's changed

- Added support for Swift external types
- Fix whitespace issues in scaffolding code breaking some versions of `rustfmt`
- Fix ruby time support
- proc-macro
  - Document (experimental) proc-macro support
  - Support fallible functions
  - Add Enum derive macro
  - Add Error derive macro

## v0.21.1 - (_2022-12-16_)

[All changes in v0.21.1](https://github.com/mozilla/uniffi-rs/compare/v0.21.0...v0.21.1).

### What's changed

- Replace checksum mechanism for function naming to give consistent results, independent of the target's endianness and bit width.
  This should have no visible effect on the outside.

## v0.21.0 - (_2022-10-14_)

[All changes in v0.21.0](https://github.com/mozilla/uniffi-rs/compare/v0.20.0...v0.21.0).

###  ⚠️ Breaking Changes ⚠️

- `uniffi_bindgen`: Renamed the `throws()` method of `Function`, `Method`, and
  `Constructor` to `throws_str()`.  Added a new `throws()` method that returns
  a boolean.

### What's changed

- Added support for exceptions in callback interface methods.
- Improved error stringifying on Kotlin and Ruby (the `message` and `to_s` methods respectively).

## v0.20.0 - (_2022-09-13_)

[All changes in v0.20.0](https://github.com/mozilla/uniffi-rs/compare/v0.19.6...v0.20.0).

###  ⚠️ Breaking Changes ⚠️

- Renamed the `uniffi_bindgen` `cydlib` argument to `lib_file`, since it can also accept static libraries

### What's changed

- The `guess_crate_root` function is now public

### What's changed
- The UDL can contain identifiers which are also keywords in Swift, except in namespace functions.

## v0.19.6 - (_2022-08-31_)

[All changes in v0.19.6](https://github.com/mozilla/uniffi-rs/compare/v0.19.5...v0.19.6).

- Fix callback interface init signature in Rust scaffolding
- Drop unused dependencies
- Update to MSRV 1.61.0

## v0.19.5 - (_2022-08-29_)

[All changes in v0.19.5](https://github.com/mozilla/uniffi-rs/compare/v0.19.4...v0.19.5).

- Fixed a small bug in the 0.19.4 release, where the extraneous `r#` was present in the HashMap generated scaffolding.

## v0.19.4 - (_2022-08-29_)

[All changes in v0.19.4](https://github.com/mozilla/uniffi-rs/compare/v0.19.3...v0.19.4).

- Implement Timestamp and Duration types in Ruby backend.
- Fixed in a bug where callback interfaces with arguments that include underscores do not get converted to camelCase on Swift.

## v0.19.3 - (_2022-07-08_)

[All changes in v0.19.3](https://github.com/mozilla/uniffi-rs/compare/v0.19.2...v0.19.3).

## v0.19.2 - (_2022-06-28_)

[All changes in v0.19.2](https://github.com/mozilla/uniffi-rs/compare/v0.19.1...v0.19.2).

- Fixed sccache issue with the `askama.toml` config file.

## v0.19.1 - (_2022-06-16_)

[All changes in v0.19.1](https://github.com/mozilla/uniffi-rs/compare/v0.19.0...v0.19.1).

### What's Changed

- Fixed the dependency from `uniffi_build` -> `uniffi_bindgen`

## v0.19.0 - (_2022-06-16_)

[All changes in v0.19.0](https://github.com/mozilla/uniffi-rs/compare/v0.18.0...v0.19.0).

###  ⚠️ Breaking Changes ⚠️
- breaking for external binding generators, the `FFIType::RustArcPtr` now includes an inner `String`. The string represents the name of the object the `RustArcPtr` was derived from.
- Kotlin exception names are now formatted as strict UpperCamelCase.  Most names shouldn't change, but names that use one word with all caps will be affected (for example `URL` -> `Url`, `JSONException` -> `JsonException`)

### What's changed
- The UDL can contain identifiers which are also keywords in Rust, Python or Kotlin.

## v0.18.0 - (_2022-05-05_)

[All changes in v0.18.0](https://github.com/mozilla/uniffi-rs/compare/v0.17.0...v0.18.0).

### ⚠️ Breaking Changes ⚠️

- When custom types are used in function/method signatures UniFFI will now use
  the UDL name for that type and create a typealias to the concrete type.  In the
  [URL example](https://mozilla.github.io/uniffi-rs/udl/custom_types.html#custom-types-in-the-bindings-code),
  this means the type will be now appear on Kotlin as `Url` rather than `URL`.
  Any existing code should continue to work because of the typealias, but this
  might affect your generated documentation and/or code completion.
- For Python libraries the native library is now loaded from an absolute path. The shared library (`*.dll` on Windows, `*.dylib` on macOS and `.so` on other platforms) must be placed next to the Python wrapper code.

### What's changed

- Allow record types with arbitrary key types
  - Record types can now contain any hashable type as its key. This is implemented for Kotlin, Python and Swift
- Python
  - Added support for default values in dictionaries
  - Generated Python code is now annotated to avoid mypy type checking
- Kotlin
  - Added external type support
- Swift
  - Fix test helper code to work with Swift 5.6

## v0.17.0 - (_2022-02-03_)

[All changes in v0.17.0](https://github.com/mozilla/uniffi-rs/compare/v0.16.0...v0.17.0).

### ⚠️ Breaking Changes ⚠️

- Wrapped types have been renamed custom types.
   - The UDL attribute is now `[Custom]` instead of `[Wrapped]`
   - The trait name has been renamed to `UniffiCustomTypeConverter` from `UniffiCustomTypeWrapper`
   - The method names of that trait have been renamed to `into_custom()` / `from_custom()` instead of `wrap()` and `unwrap()`
   - The associated type has been renamed to `Builtin` instead of `Wrapped`

### What's Changed

- Custom types (formerly wrapped) now can be configured on the bindings side as
  well as the scaffolding side.  See the "Custom Types" section of the manual
  for details.
- Kotlin now prefixes more local UniFFI variables with the `_` char to avoid
  conflicts with user-defined names.
- Updated Kotlin to use the `FfiConverter` pattern (#1144)
- Documentation updates: Added a doc comparing UniFFI to Diplomat.  Added a
  README note describing the foreign languages we currently support.
- Fixed `RustCallStatus.__str__` implementation on Python
- Fixed the version numbers in the CHANGELOG compare links.

## v0.16.0 - (_2021-12-15_)

[All changes in v0.16.0](https://github.com/mozilla/uniffi-rs/compare/v0.15.2...v0.16.0)

### ⚠️ Breaking Changes ⚠️

- Error handling when converting custom types has been updated. If your `wrap()`
  function returns an `Err`, in some cases it now [may not panic but instead
  return the error declared by the function](https://mozilla.github.io/uniffi-rs/udl/ext_types_wrapped.html#error-handling-during-conversion).

### What's Changed

- Python: Added Callback Interface support
- Swift bindings can now omit argument labels in generated functions using `omit_argument_labels = true` in the configuration.

## v0.15.2 - (_2021-11-25_)

### What's Changed
- Kotlin now generates valid code for optional timestamps/durations.

[All changes in v0.15.2](https://github.com/mozilla/uniffi-rs/compare/v0.15.1...v0.15.2).

## v0.15.1 (_2021-11-23_)

(Note that v0.15.0 was accidentally published, then yanked. v0.15.1 should be used instead)

### ⚠️ Breaking Changes ⚠️
- Previously, an interface which didn't declare a constructor got a default one anyway, making it
  impossible to decline to provide one. This is no longer true, so if your interface wants a
  constructor, you must add one explicitly.

### What's Changed
- Kotlin and Swift, like Python, now support [simple "wrapped" types](https://mozilla.github.io/uniffi-rs/udl/ext_types_wrapped.html).

- The Python backend has been refactored to more closely match the other backends, but this
  should be invisible to consumers.

- The Swift and Kotlin backends have had minor tweaks.

- The kotlin backend now explicitly checks for a null pointer in cases where it
  should be impossible to help us diagnose issues seen in the wild. See #1108.

[All changes in v0.15.1](https://github.com/mozilla/uniffi-rs/compare/v0.14.1...v0.15.1).

## v0.14.1 (_2021-10-27_)

### ⚠️ Breaking Changes ⚠️
- The `build_foreign_language_testcases!` macro now takes an array of UDL files as the
  first argument.

### What's Changed

- Swift: Added Callback Interface support
- Swift: Refactored codegen to better match Kotlin / Unit of Code
- Kotlin: Added some defensive programming around `RustBufferBuilder.discard()`

[All changes in v0.14.1](https://github.com/mozilla/uniffi-rs/compare/v0.14.0...v0.14.1).

## v0.14.0 (_2021-08-17_)

[All changes in v0.14.0](https://github.com/mozilla/uniffi-rs/compare/v0.13.1...v0.14.0).

### ⚠️ Breaking Changes ⚠️
- The Rust implementations of all `dictionary`, `enum` or `error` types defined in UDL must be
  public. If you see errors like:
    `private type <type-name> in public interface`
  or similar, please declare the types as `pub` in your Rust code.

- Errors declared using the `[Error] enum` syntax will now expose the error string from
  Rust to the foreign language bindings. This reverts an unintended change in behaviour
  from the v0.13 release which made the error message inaccessible.

### What's Changed

- You can now use external types of various flavours - see
  [the fine manual](https://mozilla.github.io/uniffi-rs/udl/ext_types.html)

- An environment variable `UNIFFI_TESTS_DISABLE_EXTENSIONS` can disable foreign language bindings
  when running tests. See [the contributing guide](./contributing.md) for more.

## v0.13.1 (_2021-08-09_)

[All changes in v0.13.1](https://github.com/mozilla/uniffi-rs/compare/v0.13.0...v0.13.1).

### What's Changed

- Fixed an accidental regression in v0.13.0 where errors were no longer being coerced
  to the correct type via `Into`. If the UDL declares a `[Throws=ExampleError]` function
  or method, the underlying implementation can now return anything that is `Into<ExampleError>`,
  matching the implicit `Into` behavior of Rust's `?` operator.
- Fixed an accidental regression in v0.13.0 where the generated Rust scaffolding assumed
  that the `HashMap` type would be in scope. It now uses fully-qualified type names in order
  to be more robust.

## v0.13.0 (_2021-08-09_)

[All changes in v0.13.0](https://github.com/mozilla/uniffi-rs/compare/v0.12.0...v0.13.0).

### ⚠️ Breaking Changes ⚠️
- UniFFI no longer has ffi-support as a dependency.  This means it handles
  panic logging on its own.  If you previously enabled the `log_panics` feature
  for `ffi-support`, now you should enable it for `uniffi`.
- The Swift bindings now explicitly generate two separate Swift modules, one for
  the high-level Swift code and one for the low-level C FFI. This change is intended
  to simplify distribution of the bindings via Swift packages, but brings with it
  some changes to the generated file layout.
  - For an interface namespace "example", we now generate:
    - A bridged C module named "exampleFFI" containing the low-level C FFI,
      consisting of an `exampleFFI.h` file and matching `exampleFFI.modulemap`
      file. The name can be customized using the `ffi_module_name` config option.
    - A Swift module named "example" containing the high-level Swift bindings,
      which imports and uses the low-level C FFI. The name can be customized using
      the `module_name` config option.
- Python timestamps will now be in UTC and timezone-aware rather than naive.
- Kotlin exceptions names will now replace a trailing "Error" with "Exception"
  rather than appending the string (FooException instead of FooErrorException)
- JNA 5.7 or greater is required for Kotlin consumers

### What's Changed

- Both python and ruby backends now handle U16 correctly.
- Error variants can now contain named fields, similar to Enum variants
- Replaced the `ViaFfi` trait with the `FfiConverter` trait.  `FfiConverter` is
  a more flexible version of `ViaFfi` because it can convert any Rust
  type to/from an Ffi type, rather than only Self.  This allows for using
  UniFFI with a type defined in an external crate.

## v0.12.0 (_2021-06-14_)

[All changes in v0.12.0](https://github.com/mozilla/uniffi-rs/compare/v0.11.0...v0.12.0).

### What's New

- It is now possible to use Object instances as fields in Records or Enums, to pass them as arguments,
  and to return them from function and method calls. They should for the most part behave just like
  a host language object, and their lifecycle is managed transparently using Rust's `Arc<T>` type.
    - Reference cycles that include Rust objects will not be garbage collected; if you cannot avoid
      creating reference cycles you may need to use Rust's `Weak<T>` type to help break them.
    - In the **Kotlin** bindings, Object instances must be manually freed by calling their `destroy()`
      method or by using their `.use` block helper method. Records or Enums that *contain* an Object
      instance now also have a `destroy()` method and must be similarly disposed of after use.

### What's Changed

- Kotlin objects now implement `AutoCloseable` by default; closing an object instance is equivalent
  to calling its `destroy()` method.

## v0.11.0 (_2021-06-03_)

[All changes in v0.11.0](https://github.com/mozilla/uniffi-rs/compare/v0.10.0...v0.11.0).

### ⚠️ Breaking Changes ⚠️

- All interface implementations must now be `Sync + Send`, and Rust will give a compile-time error
  if they are not. This makes the `[Threadsafe]` annotation redundant, so it is now deprecated and
  will be removed in a future release. More details on the motivation for this change can be found
  in [ADR-0004](https://github.com/mozilla/uniffi-rs/blob/main/docs/adr/0004-only-threadsafe-interfaces.md).

### What's Changed

- Swift structs and Kotlin data classes generated from `dictionary` are now mutable by default:
  - **Swift** now uses `var` instead of `let`
  - **Kotlin** now uses `var` instead of `val`
- Kotlin objects can now safely have their `destroy()` method or `.use` block execute concurrently
  with other method calls. It's recommended that you *not* do this, but if you accidentally do so,
  it will now work correctly rather than triggering a panic in the underlying Rust code.

## v0.10.0 (_2021-05-26_)

[All changes in v0.10.0](https://github.com/mozilla/uniffi-rs/compare/v0.9.0...v0.10.0).

### ⚠️ Breaking Changes ⚠️

- Two new built-in datatypes have been added: the `timestamp` type for representing moments in
  time, and the `duration` type for representing a difference between two timestamps. These
  mirror the `std::time::{SystemTime, Duration}` types from Rust. Thanks to @npars for
  contributing this feature!
    - This is a breaking change as it may conflict with user-declared `timestamp` or
      `duration` types in existing `.udl` files.

### What's New

- A new **Ruby** codegen backend has been added. You can now call `uniffi-bindgen -l ruby` to
  generate a Ruby module that wraps a UniFFI Rust component. Thanks to @saks for contributing
  this backend!
    - When running `cargo test` locally, you will need a recent version of Ruby and
      the `ffi` gem in order to successfully execute the Ruby backend tests.
- Threadsafe Object methods can now use `self: Arc<Self>` as the method receiver in the underlying
  Rust code, in addition to the default `self: &Self`. To do so, annotate the method with
  `[Self=ByArc]` in the `.udl` file and update the corresponding Rust method signature to match.
  This will not change the generated foreign-language bindings in any way but may be useful for
  more explicit management of Object references in the Rust code.

### What's Changed

- **Kotlin:** Fixed buggy codegen for optional primitive types like `i32?`; on earlier versions
  this would generate invalid Kotlin code which would fail to compile.

## v0.9.0 (_2021-05-21_)

[All changes in v0.9.0](https://github.com/mozilla/uniffi-rs/compare/v0.8.0...v0.9.0).

### ⚠️ Breaking Changes ⚠️

- Support for non-`[Threadsafe]` interfaces has been deprecated. A future release will require that
  all interface implementations be `Sync + Send`, making the `[Threadsafe]` annotation redundant.

### What's Changed

- Errors when parsing a `.udl` file are now marginally more useful (they're still not great, but they're better).
- Generated code should now be deterministic between runs with the same input file and version of UniFFI.
  Previously, the generated code could depend on the iteration order of an internal hash table.
- **Swift:** Generated Swift Enums now conform to `Hashable` by default.
- **Swift:** There are now additional docs on how to consume the generated Swift bindings via XCode.

## Previous releases.

We did not maintain a changelog for previous releases.
