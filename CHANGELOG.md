<!-- The sections in this file are intended to be managed automatically by `cargo release` -->
<!-- See https://github.com/sunng87/cargo-release/blob/master/docs/faq.md#maintaining-changelog for details -->
<!-- Unfortunately that doesn't currently work in a workspace, so for now we update it by hand: -->
<!--   * Replace `[[UnreleasedVersion]]` with `vX.Y.Z` -->
<!--   * Replace `[[ReleaseDate]]` with `YYYY-MM-DD` -->
<!--   * Replace `...HEAD` with `...vX.Y.Z` -->
<!--   * Insert a fresh copy of the templated bits under the `next-header` comment  -->

<!-- next-header -->

## [[UnreleasedVersion]] (_[[ReleaseDate]]_)

[All changes in [[UnreleasedVersion]]](https://github.com/mozilla/uniffi-rs/compare/v0.10.0...HEAD).

### What's Changed
 - Swift structs and Kotlin data classes generated from `dictionary` are now mutable by default:
  - **Swift** now uses `var` instead of `let`
  - **Kotlin** now uses `var` instead of `val`

### ⚠️ Breaking Changes ⚠️

- All interface implementations must now be `Sync + Send`, and Rust will give a compile-time error
  if they are not. This makes the `[Threadsafe]` annotation redundant, so it is now deprecated and
  will be removed in a future release. More details on the motivation for this change can be found
  in [ADR-0004](https://github.com/mozilla/uniffi-rs/blob/main/docs/adr/0004-only-threadsafe-interfaces.md).

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
