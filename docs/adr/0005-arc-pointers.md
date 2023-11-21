# Use raw `Arc<T>` pointers to pass objects across the FFI

* Status: accepted
* Deciders: mhammond, rfkelly
* Consulted: travis, jhugman, dmose
* Date: 2021-04-19

Discussion and approval: [PR 430](https://github.com/mozilla/uniffi-rs/pull/430).

Technical Story: [Issue 419](https://github.com/mozilla/uniffi-rs/issues/419).

Prototype: [PR 420](https://github.com/mozilla/uniffi-rs/pull/420).

## Context and Problem Statement

UniFFI currently manages object instances using the `HandleMap` struct in the ffi-support crate.
This means that external consumers of UniFFI-wrapped interfaces never see
any pointers to structs - instead, they get what is (roughly) an index into
an array, with the struct being stored in (and owned by) that array.

This has a number of safety characteristics which are particularly important for
hand-written FFI interfaces, but it does cause some issues in evolving UniFFI in
directions we consider important. In addition to the slight performance overhead,
the use of `HandleMap`s makes it difficult to support:

* Passing object instances as arguments ([#40](https://github.com/mozilla/uniffi-rs/issues/40)).
  Getting objects out of a `HandleMap` involves a closure, so accepting multiple
  object-typed arguments would involve code-generating nested closures.
* Returning object instances from functions ([#197](https://github.com/mozilla/uniffi-rs/issues/197)).
  Does the returned object already exist in the handlemap? If so, what is its handle?
  How will we manage the lifetime of multiple references to the object?

These restrictions mean that UniFFI's `Object` type is currently only suitable
as the `self` argument for method calls, and is forbidden in argument position,
as record fields, etc.

This ADR considers ways to evolve the handling of object instances and their
lifetimes, so that references to structs can be used more widely than currently allowed.

## Decision Drivers

* We desire the ability to have more flexible lifetimes for object interfaces, so
  they can be stored in dictionaries or other interfaces, and be returned by
  functions or methods other than constructors.

* We would like to keep the UniFFI implementation as simple as possible while
  providing a suitable degree of safety - in particular, a promise that it
  should be impossible to misuse the generated bindings in a way that triggers
  Rust's "undefined behavior" or otherwise defeats Rust's safety
  characteristics and ownership model (and in particular, avoiding things like
  use-after-free issues).

* We would like to keep the overhead of UniFFI as small as possible so that it
  is a viable solution to more use-cases.

## Considered Options

* **[Option 1] We extend the `HandleMap` abstraction to track lifetimes and support easier codegen**

  This would involve deciding  on how we want to track lifetimes (eg, via a reference counting or garbage
  collection) and actually building it.

* **[Option 2] We replace `HandleMap<T>` with raw pointers to Rust's builtin `Arc<T>`**

  We replace the use of HandleMaps with Rust `Arc<>`, using `Arc::into_raw` to pass
  values to the foreign-language code and `Arc::from_raw` to receive them back in Rust.

* **[Option 3] We replace `HandleMap<T>` with raw pointers to a special-purpose reference container**

  We replace the use of HandleMaps with something like [triomphe::Arc](https://docs.rs/triomphe/0.1.2/triomphe/),
  that is specifically intended for use in FFI code, using `Arc::into_raw` to pass values to the foreign-language
  code and `Arc::from_raw` to receive them back in Rust.

## Decision Outcome

Chosen option:

* **[Option 2] We replace `HandleMap<T>` with raw pointers to Rust's builtin `Arc<T>`**

This decision is taken because:

* We believe the additional safety offered by `HandleMap`s is far less
  important for this use-case, because the code using these pointers is
  generated instead of hand-written.

* Correctly implementing better lifetime management in a thread-safe way is not
  trivial and subtle errors there would defeat all the safety mechanisms the
  `HandleMap`s offer. Ultimately we'd just end up reimplementing `Arc<>` anyway,
  and the one in the stdlib is far more likely to be correct.

* There are usability and familiarity benefits to using the stdlib `Arc<>` rather
  than a special-purpose container like `triomphe::Arc`, and the way we currently
  do codegen means we're unlikely to notice any potential performance improvements
  from using a more specialized type.

### Positive Consequences

* There will be less overhead in our generated code - both performance overhead
  and cognitive overload - it will be much easier to rationalize about how
  the generated code actually works and performs.

### Negative Consequences

* Errors in our generated code might cause pointer misuse and lead to "use
  after free" type issues.

* Mis-use of generated APIs may be able to create reference cycles between Rust
  objects that cannot be deallocated, and consumers coming from a garbage-collected
  language may assume that such cycles will be collected.

## Pros and Cons of the Options

### [Option 1] We extend the `HandleMap` abstraction to track lifetimes and support easier codegen.

* Good, because raw pointers aren't handed out anywhere.

* Bad, because we need to reimplement safe reference counting or garbage
  collection.

* Bad, because code generation is likely to remain somewhat complex.

Overall, this option is dispreferred because it will involve writing significant new and complex code,
the safety benefits of which will be quite marginal in practice.

### [Option 2] We replace `HandleMap<T>` with raw pointers to Rust's builtin `Arc<T>`

* Good, because the code generated by UniFFI is clearer and easier to understand.

* Good, because we can reuse the Rust standard library and have confidence in
  its implementation.

* Bad, because handing raw pointers around means bugs in the generated code or
  intentional misuse of the bindings might cause vulnerabilities.

Overall, this option is preferred because it achieves the goals while reducing both the
performance overheads of the generated code, and the cognitive overheads of maintaining
the tool.

### [Option 3] We replace `HandleMap<T>` with raw pointers to a special-purpose reference container

* Good, because the code generated by UniFFI is clearer and easier to understand.

* Good, because we can reuse an existing well-tested container type like `triomphe:Arc`.

* Good, because the special-purpose container type may be more performant than the
  default implementation in the Rust stdlib.

* Bad, because handing raw pointers around means bugs in the generated code or
  intentional misuse of the bindings might cause vulnerabilities.

* Bad, because the special-purpose container may "leak" into the implementation of the UniFFI-wrapped
  Rust code, adding cognitive overhead for consumers.

Overall, this option is dispreferred due to additional cognitive overhead for consumers.
The potential performance improvements seem likely to be lost in amongst the many other
sources of overhead in our current generated code.

We may reconsider this decision if future profiling shows the use of stdlib `Arc` to be a bottleneck.

## Links

* Thom discusses this a bit in [this issue](https://github.com/mozilla/uniffi-rs/issues/244)
  and agrees with the assertion that raw pointer make sense when all
  the relevant code is generated.

* Ryan discusses his general approval for this approach in [this issue](https://github.com/mozilla/uniffi-rs/issues/419)
  and the [PR for this ADR](https://github.com/mozilla/uniffi-rs/pull/430)
