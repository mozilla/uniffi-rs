# Use handle map handles to pass objects across the FFI

* Status: proposed
* Deciders:
* Consulted:

Discussion and approval: 

ADR-0005 discussion: [PR 430](https://github.com/mozilla/uniffi-rs/pull/430).

## Context and Problem Statement

UniFFI currently passes objects from Rust to the foreign side by leaking an Arc reference into a word-sized opaque pointer and passing it across the FFI.
The basic approach uses `Arc::into_raw` / `Arc::from_raw` and was chosen in [ADR-0005](./0005-arc-pointers.md) for several reasons:

  1. Clearer generated code.
  2. Ability to pass objects as arguments (https://github.com/mozilla/uniffi-rs/issues/40).
     This was deemed difficult to do with the existing codegen + HandleMap.
  3. Ability to track object identity (https://github.com/mozilla/uniffi-rs/issues/197).  If two function calls return the same object, then this should result in an identical object on the foreign side.
  4. Increased performance.

Recently, this approach was extended to work with unsized types (`Arc<dyn Trait>`), which are normally wide pointers (i.e double-word sized).
For these types, we box the Arc to create `Box<Arc<dyn Trait>>`, then leak the box pointer.
This results in a regular, word-sized, pointer since `Arc<dyn Trait>` is sized (2 words) even when `dyn Trait` is not.

Now that we have several years of experience, it's a good time to revisit some of the reasoning in ADR-0005 because it seems like we're not getting the benefits we wanted:

* The code that deals with these isn't so clear, especially when we have to deal with unsized types (for example
  the RustFuture
  [allocation](https://github.com/mozilla/uniffi-rs/blob/fbc6631953a889c7af6e5f1af94de9242589b75b/uniffi_core/src/ffi/rustfuture/mod.rs#L56-L63) / [dellocation](https://github.com/mozilla/uniffi-rs/blob/fbc6631953a889c7af6e5f1af94de9242589b75b/uniffi_core/src/ffi/rustfuture/mod.rs#L124-L125) or the similar code for trait interfaces).
* The codegen has progressed and it would be easy to support `[2]`.
  We could simply `clone` the handle as part of the `lower()` call.
* We've never implemented the reverse identity map needed for `[3]`.
  The `NimbusClient` example given in https://github.com/mozilla/uniffi-rs/issues/419 would still fail today.
  Given that there has been little to no demand for this feature, this should be changed to a non-goal.
* The performance benefit decreases when discussing unsized types which require an additional layer of boxing.
  In that case, instead of a strict decrease in work, we are trading a `HandleMap` insertion for a Box allocation.
  This is a complex tradeoff, with the box allocation likely being faster, but not by much.

Furthermore, practice has shown that dealing with raw pointers makes debugging difficult, with errors often resulting in segfaults or UB.
Dealing with any sort of FFI handle is going to be error prone, but at least with a handle map we can generate better error messages and correct stack traces.
There are also more error modes with this code.

### Safety

ADR-0005 says "We believe the additional safety offered by `HandleMap`s is far less important for this use-case, because the code using these pointers is generated instead of hand-written."

While it's certainly true safety benefits matter less for generated code, it's also true that UniFFI is much more complex now then when ADR-0005 was decided.
We have introduced callback interfaces, trait interfaces, Future handles for async functions, etc.
All of these introduce additional failure cases, for example #1797, which means that relatively small safety benefits are more valuable.

### Foreign handles

A related question is how to handle handles to foreign objects that are passed into Rust.
However, that question is orthogonal to this one and is out-of-scope for this ADR.

## Considered Options

### [Option 1] Continue using raw Arc pointers to pass Rust objects across the FFI

Stay with the current status quo.

### [Option 2] Use the old `HandleMap` to pass Rust objects across the FFI

We could switch back to the old handle map code, which is still around in the [ffi-support crate](https://github.com/mozilla/ffi-support/blob/main/src/handle_map.rs).
This implements a relatively simple handle-map that uses a `RWLock` to manage concurrency.

See [../handles.md] for details on how this would work.

Handles are passed as a `u64` values, but they only actually use 48 bits.
This works better with JS, where the `Value` type only supports integers up to 53-bits wide.

### [Option 3] Use a `HandleMap` with more performant/complex concurrency strategy

We could switch to something like the [handle map implementation from #1808](https://github.com/bendk/uniffi-rs/blob/d305f7e47203b260e2e44009e37e7435fd554eaa/uniffi_core/src/ffi/slab.rs).
The struct in that code was named `Slab` because it was inspired by the `tokio` `slab` crate.
However, it's very similar to the original UniFFI `HandleMap` and this PR will call it a `HandleMap` to follow in that tradition.

See [../handles.md] for details on how this would work.

### [Option 4] Use a 3rd-party crate to pass Rust objects across the FFI

We could also use a 3rd-party crate to handle this.
The `sharded-slab` crate promises lock-free concurrency and supports generation counters.

## Decision Drivers

## Decision Outcome

???

## Pros and Cons of the Options

### [Option 1] Continue using raw Arc pointers to pass Rust objects across the FFI

* Good, because it has the fastest performance, especially for sized types.
* Good, because it doesn't require code changes.
* Bad, because it's hard to debug errors.

### [Option 2] Use the original handle map to pass Rust objects across the FFI

* Good, because it's easier to debug errors.
* Bad, because it requires a read-write lock.
  In particular, it seems bad that `insert`/`remove` can block `get`.
* Good, because it works better with Javascript
* Good, because it works with any type, not just `Arc<T>`.
  For example, we might want to pass a handle to a [oneshot::Sender](https://docs.rs/oneshot/latest/oneshot/) across the FFI to implement async callback interface methods.

### [Option 3] Use a handle map with a simpler concurrency strategy

* Good, because it's easier to debug errors.
* Good because `get` doesn't require a lock.
* Bad because `insert` and `remove` requires a lock.
* Bad, because it requires consumers to depend on `append-only-vec`.
  However, this is a quite small crate.
* Good, because it works better with Javascript
* Good, because it works with any type, not just `Arc<T>`.

### [Option 4] Use a 3rd-party crate to pass Rust objects across the FFI

* Good, because it's easier to debug errors.
* Bad, because it requires consumers to take this dependency.
* Bad, because it makes it harder to implement custom functionality.
  For example, supporting clone to fix https://github.com/mozilla/uniffi-rs/issues/1797 or adding a foreign bit to improve trait interface handling.
