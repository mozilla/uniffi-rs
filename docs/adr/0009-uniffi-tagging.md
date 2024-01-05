# Handling `UniffiTag` and using types from other crates

* Status: proposed
* Deciders: Uniffi developers
* Date: 2024-01-05

## Context and Problem Statement

UniFFI's various FFI-related traits `FfiConverter`, `Lift`, `Lower`, `LiftReturn`, `LowerReturn`, etc. all have a generic "tag" parameter.
The code in `uniffi_macros::setup_scaffolding` generates a zero-sized `crate::UniffiTag` type for use in that parameter in other generated code.
For example, when lowering a type we use `<#type as Lower<crate::UniffiTag>>::lower`.

The reason for this is the so-called "Orphan rule", which prevents generated code from implemented a remote trait on a remote type.
If the FFI traits did not have a generic parameter, then it would be impossible to generate code that implements those traits for a type defined in another crate.
See [ADR-0006 Wrapping types](./0006-wrapping-types.md) for more discussion.

For local types we have a choice: implement only for the local tag (`impl Lift<crate::UniffiTag> for MyType`) or implement it for all tags (`impl<UT> Lift<UT> for MyType)`).
This choice mainly affects other crates that also want to use that type.
If `crate_a` implements the traits for their local tag and `crate_b` uses that type, then users must take some explicit action to implement the FFI traits for `crate_b::UniffiTag`
If `crate_a` implements the traits for all tags, then users will normally not need to take any explicit action.
However, in the case where `crate_a` is implementing the trait for a remote type, then they will.
This is quite confusing in the abstract, hopefully the example below clears things up.

Currently, UDL-based generation uses the former approach and proc-macros use the latter.
It would be better to pick one approach and stick with it.

One additional option is to remove the tags parameter from the ffi traits altogether.
This would simplify the code generation significantly, but it's not clear how to handle remote types.

### Example

- `crate_a` depends on the `uuid` crate and uses the [Uuid](https://docs.rs/uuid/latest/uuid/struct.Uuid.html) type in their exported API.
  It uses a [custom type](https://mozilla.github.io/uniffi-rs/proc_macro/index.html#the-unifficustom_type-and-unifficustom_newtype-macros) that converts the URL to a string for the foreign bindings.
  Custom types are common in this situation, since the remote type was not designed with UniFFI and foreign-language consumption in mind.
  Note: this is currently impossible to do with proc-macros since they always generate a blanket impl for all UniffiTags, but let's assume that a method was added to only generate an impl for the local tag.
- `crate_a` defines/exports a `Record` type, which is a UniFFI dict/record that has a `uuid` field.
- `crate_b` depends on `crate_a` and `uuid` and defines a function `lookup_record(uuid: Uuid) -> Record`

We have several options for handling this situation.

### Always implement traits for the local tag only

- `crate_a` implements the FFI traits for its local `UniffiTag` for both `Uuid` and `Record`.
- `crate_b` needs to take explicit action to implement the FFI traits for its `UniffiTag` in order to use them in its API.
  For example:

```rust

use crate_a::{Record, Uuid};

uniffi::use!(crate_a, Record)
uniffi::use!(crate_a, Uuid)

#[uniffi::export]
pub fn lookup_record(uuid: Uuid) -> Record {
    ...
}
```

Note: right now these macros are named `use_udl_object`, etc.
Once proc-macros can implement traits for remote types we probably should rename them to something not specific to UDL, but the exact name is out of scope for this ADR.
Also out of scope is allowing users to not have to specify if the kind of type it is, however this should be possible now that we have library mode and know the metadata for all crates.

### Implement traits for all tags when possible


- `crate_a` implements the FFI traits for its local `UniffiTag` for `Uuid`
- `crate_a` implements the FFI traits for all tags for `Record`.
  This is allowed since the type is local to `crate_a`.
- `crate_b` only needs to take explicit action to use `Uuid` since it's a remote type for `crate_a`.

```rust

use crate_a::{Record, Uuid};

uniffi::use!(crate_a, Uuid)

#[uniffi::export]
pub fn lookup_record(uuid: Uuid) -> Record {
    ...
}
```

### Remove the `UniffiTag` parameter from the FFI traits, don't allow implementing them for remote types

In this scenario, crates would not be able to implement the FFI traits for `Uuid` at all.
This means no crates could use them directly in their exported APIs.
Consumers could work around this by defining a separate API layer that has a lot of `into()`s.

`crate_a`:

```rust

/// Record type used in most of the code
pub struct Record {
    id: Uuid,
    ...
}

/// Record type used in the API layer
#[derive(uniffi::Record))
pub struct FfiRecord {
    id: String,
    ...
}

impl From<Record> for FfiRecord {
    ...
}

impl From<FfiRecord> for Record {
    ...
}
    
```

`crate_b`:

```rust
use crate_a::{FfiRecord, Record, Uuid};

pub fn lookup_record(uuid: Uuid) -> Record {
    ...
}

pub mod ffi {
    #[uniffi::export]
    pub fn lookup_record(uuid: String) -> FfiRecord {
        super::lookup_record(uuid.into()).into()
    }
}
```

## Decision Drivers

- Using one system for UDL and a different one for proc-macros is overly complicated and therefore not discussed as an option.
- TODO: add more as we discuss this.

## Considered Options

### [Option 1] Always implement FFI traits for the local tag only

### [Option 2] Implement FFI traits for all tags for non-remote tags

### [Option 3] Remove the UniffiTag parameter from the FFI traits

## Pros and Cons of the Options

### [Option 1] Always implement FFI traits for the local tag only

- Good, because it's easier to explain.
  Telling users "you always have to add `uniffi::use_*!`" to use a type from another crate is easier than saying you sometimes you need it and sometimes you don't.

### [Option 2] Implement FFI traits for all tags for non-remote tags

- Good, because it's less typing
- Good, because it's the simplest system when remote types are not involved.
  If all types come from the user's crates or the 3rd-party crates, then users don't need to take any actions and can just use the types naturally.

### [Option 3] Remove the UniffiTag parameter from the FFI traits

- Good, because it simplifies the code generation.
- Bad, because it requires consumers to write boilerplate code.
- Bad, because having both regular and FFI API can get confusing.  In particular, it can be hard to keep the names of everything straight.

## Decision Outcome
