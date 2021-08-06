# Replace ViaFfi with FfiConverter to handle external types

* Status: proposed
* Deciders: bendk, rfkelly, mhammond
* Consulted: jhugman, jan-erik, travis
* Date: 2021-08-06

Discussion : [PR 1001](https://github.com/mozilla/uniffi-rs/pull/1001).

## Context and Problem Statement

UniFFI currently cannot support types from external crates because of Rust's
orphan rule prevents implementing `ViaFfi`.  The orphan rules specifies that
the trait needs to be in the `uniffi` crate or the external crate, but we are
generating the scaffolding code for the crate being uniffied.

Because of the orphan rule, any solution to this must be implemented using a
new struct that we create in the scaffolding code.  The question is how to use
that struct to lift/lower the external type that we actually want to use.

## Decision Drivers

* We want a solution that's general and easy to extend.  The solution should
  enable, or at least not conflict with, other features we're planning to add:
    * Importing types from another UDL
    * Wrapping primitive types with a Rust type (for example String with Guid)
    * Extension types that wrap a primitive on both the Rust and foreign
      language side (JSON)

## Considered Options

* **[Option 1] Extend the template code to wrap the type**

  * In the Record/Enum/Object/Error code, we would use the newtype pattern to
    wrap the external type (`struct WrapperType(ExternalType)`)

  * In the filters functions we generate code to wrap lift/lower/read/write.
    For example the lower_rs filter could output `External_type(x).lower()` to
    lower `WrapperType`.

* **[Option 2] Replace the `ViaFfi` trait with something more generic**

  * We define `FfiConverter` which works almost the same as `ViaFfi` except
    instead of always converting between `Self` and `FfiType`, we define a
    second associated type `RustType` and convert between `RustType` and
    `FfiType`.

  * For each user-defined type (Record, Error, Object, and Enum), we create a
    new unit struct and set `RustType` to the type.  This handles external types
    without issues since we're implementing `FfiConverter` on our on struct.
    The orphan rule doesn't apply to associated types.


## Decision Outcome

Chosen option:

* **[Option 2] Replace the `ViaFfi` trait with something more generic**

This decision is taken because:

* This solution has a simpler implementation.  The template code to generate the
  above example is not trivial.  It would need to work with any variable name,
  wrapping a return value, and also with recursive types (for example Option
  would need to work with wrapped structs).

* We believe it will make it easier to implement the other wrapping-style
  features mentioned above.  One sign of this was the CallbackInterface code,
  which converts it's type to `Box<dyn CallbackInterfaceTrait>`.  The
  `FfiConverter` trait was able to implement this, removing the current
  lift/lower/read/write template code and fixing a bug with `Option<>`.

### Positive Consequences

* The wrapper/adapter pattern should be easier to implement in the future.

### Negative Consequences

* It requires a shift in all of our mental models.  `FfiConverter` seems like
  it's basically the same as `ViaFfi`, but the small code change actually
  fundamentally changes how things work.  This motivated us to change the name
  of the trait.
