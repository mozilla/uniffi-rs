<!-- The sections in this file are managed automatically by `cargo release` -->
<!-- See our [internal release process docs](docs/release-process.md) and for more general -->
<!-- guidance, see https://github.com/sunng87/cargo-release/blob/master/docs/faq.md#maintaining-changelog -->

<!-- next-header -->

## [[UnreleasedVersion]] - (_[[ReleaseDate]]_)

[All changes in [[UnreleasedVersion]]](https://github.com/mozilla/uniffi-rs/compare/v0.9.0...HEAD).

## v0.16.0 - (_2021-12-15_)

[All changes in v0.16.0](https://github.com/mozilla/uniffi-rs/compare/v0.9.0...v0.16.0)

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

[All changes in v0.15.2](https://github.com/mozilla/uniffi-rs/compare/v0.9.0...v0.15.2).

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
  matching the implicit `Into` behavoir of Rust's `?` operator.
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
