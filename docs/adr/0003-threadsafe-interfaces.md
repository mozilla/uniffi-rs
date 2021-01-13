# Let consumers opt out of HandleMap locking if their interface is threadsafe

* Status: proposed
* Deciders: rfkelly, jhugman, mhammond, dmosedale
* Date:  2021-01-13

## Context and Problem Statement

Uniffi currently uses a very coarse locking strategy for managing concurrent access to object instances,
which has caused us to accidentally ship code in a product that [blocked the main thread on network I/O](https://jira.mozilla.com/browse/SDK-157).
We need to enable finer-grained concurrency control in order to provide the desired API for a key consumer.

Currently, every interface has a corresponding [ffi_support::ConcurrentHandleMap](https://docs.rs/ffi-support/0.4.0/ffi_support/handle_map/struct.ConcurrentHandleMap.html) that is responsible for owning all instances of
that interface and for handing out references to them in a mutability-safe and threadsafe manner. This
ensures that the generated code is safe in the face of concurrent operations, but has a substantial
runtime cost: only one method call can be executed on an instance at any time. Any attempt to call an
object method while a concurrent method is already executing, will block until the previous call has completed.

The desired API for Project Nimbus includes methods that will be called synchronously from the main thread,
and hence must not block on network or disk I/O. Such an API cannot be built with uniffi as currently
implemented.

## Decision Drivers <!-- optional -->

* Enabling consumers to control the potential blocking behaviour of their generated APIs.
* Ensure safety of the generated code.
* Ship a solution in a timely manner to unblock Project Nimbus.

## Considered Options

* Option A: Do nothing, require consumers to assume that all method calls might block.
* Option B: Let consumers mark interface definitions as threadsafe to opt in to a less-locking handlemap.
* Option C: Insist that all interfaces be threadsafe and replace the handlemap with raw pointers.
* Option D: Use a less-locking handlemap and rely on calling code to behave safely.

## Decision Outcome

Chosen option:
 * **Option B: Let consumers mark interface definitions as threadsafe to opt in to a less-locking handlemap.**

The choice here comes down to safety and simplicity. By making a more-concurrency-friendly handlemap
we can maintain the current strict enforcement of Rust's mutability-safety and thread-safety guarantees,
even in the face of errors in the generated bindings. It seems to be a relatively small change, and
by making it opt-in we avoid creating busywork for other consumers who are not urgently facing this
problem.

One downside is that consumers need to opt-in to the fix, meaning that the default behavior may still
be surprising to new consumers. We'll mitigate this with docs and will consider revisiting the default
behaviour if the majority of consumers adopt the new approach.

This choice does also punt some potential performance improvements to future work, but that seems in keeping
with where we are in the project's lifecycle.

## Pros and Cons of the Options

### Option A: Do nothing, require consumers to assume that all method calls might block.

Make no changes to uniffi, and instead accept the fact that method calls are executed serially.
Document this limitation and work with consumers to update their API definitions to account for it.

* Good, because we can ship this quickly.
* Good, because we don't give up any of the safety guarantees of the current approach.
* Bad, because it makes it basically impossible to meet the needs of one of our key early consumers,
  and would force us into awkward compromises around a suboptimal API.
* Bad, because it makes uniffi less attractive to potential future consumers.
* Bad, because it keeps all the run-time overhead of the handlemap.
* Bad, because the default behaviour still has a hidden mutex, which might be a nasty surprise
  for future consumers in the same way that it was for Nimbus.

Ultimately, we want uniffi to be a tool that helps consumers deliver value, not something that
foist limitations upon them, which makes this option unattractive.

### Option B: Let consumers mark interface definitions as threadsafe to opt in to a less-locking handlemap.

Implement a variant of `ConcurrentHandleMap` that does not protect its members with a Mutex, which
is the main source of constraints on concurrent execution in the current setup. Instead, this handlemap
would only give out immutable references to its members, and would insist that its members are `Send`
and `Sync` so they can be safely accessed from multiple threads.

Introduce a new annotation to the UDL so that `interface` definitions can be declared as threadsafe.
In the generated Rust scaffolding, use the new handlemap for threadsafe interfaces but keep using the
existing `ConcurrentHandleMap` by default.

* Good, because this is a relatively small change from the current behaviour, which should be fairly
  quick to ship.
* Good, because this is a non-breaking change for consumers who don't opt in to it.
* Good, because the `Sync` and `Send` bounds on the underlying struct will help consumers
  to implement their own fine-grained concurrency control while being supported by Rust's
  compile-time guarantees.
* Good, because consumers don't have to care about this until they discover that they need
  fine-grained locking.
* Good, because it leaves the door open to further improvements (like "Option C") in the
  future if we decide that makes sense later.
* Bad, because it keeps all the run-time overhead of the handlemap.
* Bad, because now we need to maintain two handlemap variants.
* Bad, because the default behaviour still has a hidden mutex, which might be a nasty surprise
  for future consumers in the same way that it was for Nimbus.

This is the solution we have ultimately selected.

### Option C: Insist that all interfaces be threadsafe and replace the handlemap with raw pointers.

Stop using `ConcurrentHandleMap` to intermediate object access. Instead, put each object instance
in a `Box` and use `Box::into_raw` to transfer ownership of the box to the foreign language code
as a raw pointer.

Maintain safety by insisting that any struct implementing a UDL `interface` must be `Sync` and `Send`,
and by refusing to hand out mutable references to the boxed instance.

* Good, because it removes the issue as a potential footgun for all consumers.
* Good, because the `Sync` and `Send` bounds on the underlying struct will help consumers
  to implement their own fine-grained concurrency control while being supported by Rust's
  compile-time guarantees.
* Good, because it likely reduces runtime overhead and gives a small performance boost.
* Bad, because it's a breaking change for all consumers, even if they don't care about this issue.
* Bad, because it increases the amount of unsafe rust that we need to emit in the generated scaffolding.
* Bad, because we lose some additional runtime checks provided by the handlemap, such as guarding
  against passing a handle that belongs to a different datatype.
* Bad, because it's a non-trivial departure from the current approach which will take more time to QA.

This seems like a promising longer-term option, especially if we find that the majority of consumers
are opting in to the fix proposed in this ADR. But the additional complexity weights heavily against
trying to ship this as an initial fix for Project Nimbus.

### Option D: Use a less-locking handlemap and rely on calling code to behave safely.

Implement a variant of `ConcurrentHandleMap` that does not protect its members with a Mutex, which
is the main source of constraints on concurrent execution in the current setup. Instead, this handlemap
would hand out references without any runtime checks, on the assumption that the calling code is
behaving in a safe manner. Replace all currently uses of `ConcurrentHandleMap` with this new less-locking variant.

* Good, because this is a relatively small change from the current behaviour, which should be fairly
  quick to ship.
* Bad, because we lost Rust's compile-time safety guarantees.
* Bad, because it's hard to communicate to consumers what "behave safely" really means (and we might not
  even understand it ourselves).

This seems to throw away some of the key safety benefits of using Rust, which makes it a very
unattractive option.

## Implementation Sketch

### Add a `[Threadsafe]` attribute to UDL `interface` definitions.

We would advise Nimbus SDK to update their UDL definition for `NimbusClient` like so:

```idl
[Threadsafe]
interface NimbusClient {
  // existing method definitions remain unchanged
}
```

The [`uniffi_bindgen::interface::Object`](https://github.com/mozilla/uniffi-rs/blob/803bb3d79daa8ea088fb2d8f05c08ada09821986/uniffi_bindgen/src/interface/mod.rs#L722) struct
would grow a corresponding boolean `threadsafe` field and corresponding public accessor.
When [building an instance of this struct from the UDL](https://github.com/mozilla/uniffi-rs/blob/803bb3d79daa8ea088fb2d8f05c08ada09821986/uniffi_bindgen/src/interface/mod.rs#L786),
we would inspect the list of attributes to look for one named "Threadsafe" and set the field
if present. The way that we [handle the `[ByRef]` annotation on method arugments](https://github.com/mozilla/uniffi-rs/blob/803bb3d79daa8ea088fb2d8f05c08ada09821986/uniffi_bindgen/src/interface/mod.rs#L658)
will likely serve as a good example to follow.

### Implement a less-locking HandleMap variant

The `ffi_support` crate provides a basic non-locking [`HandleMap<T>`](https://docs.rs/ffi-support/0.4.0/ffi_support/handle_map/struct.HandleMap.html) struct,
and it implements `ConcurrentHandleMap` as a thin wrapper around a `RwLock<HandleMap<Mutex<T>>>`. We can make our own variant that removes the inner mutex, as a thin wrapper around a`RwLock<HandleMap<Arc<T>>>`. (The outer `RwLock` is still needed in order to guard mutations of the handlemap itself, while the `Arc` is needed so that we can quickly clone a reference and
release the lock before calling potentially-long-running methods on that reference).

Bikeshed name: `LessLockingHandleMap`, implemented under the [`uniffi::ffi` module](https://github.com/mozilla/uniffi-rs/tree/main/uniffi/src/ffi). It needs to live in the `uniffi` crate so that it can be used by the generated Rust scaffolding at runtime.

We don't need to support the entire `ConcurrentHandleMap` interface, only:
* `insert_with_output`
* `insert_with_result`
* `call_with_output`
* `call_with_result`
* `delete_u64`

The `LessLockingHandleMap<T>` struct should be capable of handing out `&T` references but should never hand
out a `&mut T`. It should also require that `T: Send + Sync`.

### Use `LessLockingHandleMap` in the Rust scaffolding for `[Threadsafe]` instances

In [ObjectTemplate.rs](https://github.com/mozilla/uniffi-rs/blob/main/uniffi_bindgen/src/templates/ObjectTemplate.rs), check the `threadsafe` property of the object definition. If it's true, use `LessLockingHandleMap` instead of `ConcurrentHandleMap` in the [lazy static that declares the handlemap for that interface](https://github.com/mozilla/uniffi-rs/blob/803bb3d79daa8ea088fb2d8f05c08ada09821986/uniffi_bindgen/src/templates/ObjectTemplate.rs#L11).

By ensuring that `LessLockingHandleMap` and `ConcurrentHandleMap` expose a similar API, this will hopefully
be a fairly minimal if-then-else that just chooses the correct name of the struct to use.

### ~~Optional: Add a `[Blocking]` annotation to methods and functions~~

(This was moved into a follow-up issue, ref [#378](https://github.com/mozilla/uniffi-rs/issues/378)).

## Links

* [Fenix Bug: Large regression in MAIN/VIEW start up](https://jira.mozilla.com/browse/SDK-157).
* [An proof-of-concept experiment in using raw pointers rather than a handlemap](https://github.com/mozilla/uniffi-rs/pull/366).
* [The final implementation of this ADR: #372](https://github.com/mozilla/uniffi-rs/pull/372)