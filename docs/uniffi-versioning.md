# The UniFFI Versioning System

UniFFI versions are tricky since both libraries and applications depend on UniFFI.  This can lead to situations where `app_foo` depends on `lib_bar` and `lib_baz`, and all of those depend on UniFFI.  In that situation, each of those crates must depend on a UniFFI version that's compatible with all the others.

All of this means that breaking changes to UniFFI are costly for our consumers.  In the situation above, if `lib_bar` upgrades UniFFI with a breaking change, then both `lib_baz` and `app_foo` would also need to upgrade UniFFI and all of these changes would need to be coordinated together.

Therefore, we want to have a system which minimizes the amount of breaking changes for consumers.

## Breaking changes and SemVer

UniFFI follows the [SemVer rules from the Cargo Book](https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility) which states "Versions are considered compatible if their left-most non-zero major/minor/patch component is the same".  Since all crates are currently on major version `0`, this means a breaking change will result in a bump of the minor version.  Once we are on major version `1` or later, a breaking change will result in a major version bump.  In the text below, these are referred to as a "breaking version bump".

## How consumers should depend on UniFFI

Crates that use UniFFI to generate scaffolding or bindings should only have a direct dependency to the `uniffi` crate, which re-exports the top-level functionality from other crates:

* Generating the scaffolding via a build script
* Generating the bindings via a CLI
* Generating the scaffolding or bindings programmatically

Because the crates only directly depend on `uniffi`, they only need to care about the `uniffi` version and can ignore the versions of sub-dependencies.  This means that breaking changes in `uniffi_bindgen` won't be a breaking change for consumers, as long as it doesn't affect the functionality listed above.

## What is a breaking change?

To expand on the previous point, here are the scenarios where `uniffi` should get a breaking version bump:

* Backward incompatible changes to the UDL/proc-macro parsing:
  * Removing a feature.
  * Changing how existing UDL/proc-macro code is handled -- for example if we changed UniFFI functions to return a `Result<>` enum rather than throwing exceptions.
  * Note: Adding new UDL or proc-macro features is not a breaking change.
* Backward incompatible changes to the FFI contract between the scaffolding and bindings code:
  * Changing how FFI functions are named.
  * Changing how FFI functions are called
  * Changing how types are represented.

## How to handle breaking changes

* Increment the minor version of `uniffi`
  * Once we get to `1.0` then this will change to be a major version bump.
* Update the `uniffi_bindgen::UNIFFI_CONTRACT_VERSION` string
