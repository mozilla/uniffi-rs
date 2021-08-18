# Update the code or update the template wrappers?

* Status: proposed
* Deciders: bendk, rfkelly, mhammond
* Consulted: jhugman, jan-erik, travis
* Date: 2021-08-06

Discussion : [PR 1001](https://github.com/mozilla/uniffi-rs/pull/1001).

## Context and Problem Statement

UniFFI was not able to support types from external crates because Rust's orphan
rule prevents implementing the `ViaFfi` trait.  In order to add support we
needed to choose between updating the `uniffi` traits or updating the
`lift_py` and `lower_py` scaffolding functions.

The same general question comes up often.  When adding new features we often
need to choose between two paths:

  * Updating the code in the target language
  * Updating the template wrapping code

This ADR discusses this particular decision and also the general pros and cons of each
path.

## Decision Drivers

* We wanted to support external crates that define new types by wrapping an
  UniFFI primitive type.  For example supporting `serde_json::Value` that wraps
  `string` or a `Handle` that wraps `int32`.  We wanted this kind of wrapping
  code to exist outside of `uniffi` to allow for more experimentation with
  wrapped types and to support types that were specific to particular libraries
  (for example the application-services `Guid` type).

## Considered Options

* **[Option 1] Extend the template code to wrap the type**

  * In the Record/Enum/Object/Error code, we would use the newtype pattern to
    wrap the external type (`struct WrapperType(ExternalType)`)

  * In the filters functions we generate code to wrap lift/lower/read/write.
    For example the lower_rs filter could output `WrapperType(x).lower()` to
    lower `WrapperType`.

* **[Option 2] Update the `uniffi` code and generalize the `ViaFfi` trait**

  * We define `FfiConverter` which works almost the same as `ViaFfi` except
    instead of always converting between `Self` and `FfiType`, we define a
    second associated type `RustType`.  `FfiConverter` converts between
    any `RustType` and `FfiType`.

  * For each user-defined type (Record, Error, Object, and Enum), we create a
    new unit struct and set `RustType` to the type.  This handles external types
    without issues since we're implementing `FfiConverter` on our on struct.
    The orphan rule doesn't apply to associated types.

   * This eliminated the `lower_rs`, `lift_rs`, `read_rs`, and `write_rs`
     filter functions.  All FFI conversions were now handled by Rust code
     directly.

## Decision Outcome

Chosen option:

* **[Option 2] Update the `uniffi` code and generalize the `ViaFfi` trait**

This decision is taken because:

* It's was relatively easy to implement wrapper types by allowing the external
  crates to add custom scaffolding code.  This code could wrap primitive types
  because all lifting/lowering/reading/writing was handled by Rust code.  If we
  had gone with option 1, then the wrapping code would need to hook into the
  template functions (`lift_rs`, `lower_rs`, etc.).  We couldn't see a simple
  way to implement this.

* Updating the code in the target language results in more readable generated
  code.  The newtype pattern makes the generated code more difficult to read,
  especially when types are wrapped in `Option<>`, `Vec<>`, etc.

* The same pattern could be used to implement wrapping on the bindings side.

### Positive Consequences

* Paved the way for wrapper types.
* Simplified the template code.

### Negative Consequences

* Implementing wrapping with template functions can lead to more direct code.
  For example, lifting a integer value is a no-op, but we still generate a
  function call to do it.  This is not an issue with Rust, since the compiler
  will optimize the call away, but it could be an issue for the bindings code.
  If we decide that it is an issue, we could go with a hybrid solution:
  generate the lifting/lowering code in the target language, but also have
  lift/lower filter functions that exist solely to optimize lifting/lowering
  simple types.
