# uniffi - a multi-language bindings generator for rust

This is a little experiment in building cross-platform components in Rust, based on things
we've learned in the [mozilla/application-services](https://github.com/mozilla/application-services)
project.

It's currently at the "basic prototype" stage - the core idea seems to work, but there are a lot of
missing details and it needs a lot of polish. We're currently focussed on making it useable for a
real shipping component.

## What?

We're interested in building re-useable components for sync- and storage-related browser
functionality - things like [storing and syncing passwords](https://github.com/mozilla/application-services/tree/main/components/logins),
[working with bookmarks](https://github.com/mozilla/application-services/tree/main/components/places) and
[signing in to your Firefox Account](https://github.com/mozilla/application-services/tree/main/components/fxa-client).

We want to write the code for these components once, in Rust. We want to easily re-use these components from
all the different languages and on all the different platforms for which we build browsers, which currently
includes JavaScript for PCs, Kotlin for Android, and Swift for iOS.

And of course, we want to do this in a way that's convenient, maintainable, and difficult to mess up.

## How?

In an aspirational world, we could get this kind of easy cross-language interop for
free using [wasm_bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) and
[webassembly interface types](https://hacks.mozilla.org/2019/08/webassembly-interface-types/) -
imagine writing an API in Rust, annotating it with some `#[wasm_bindgen]` macros,
compiling it into a webassembly bundle, and being able to import and use that bundle
from any target language, complete with a rich high-level API!

That kind of tooling is not available to shipping applications today, but that doesn't
mean we can't take a small step in that general direction while the Rust and Wasm ecosystem
continues to evolve.

Using `uniffi`, you can:

* Implement your software component as a `cdylib` crate in Rust; let's say the code is in `./src/lib.rs`.
* Specify the desired component API using an abstract *Interface Definition Language*, in a separate file like `./src/lib.idl`.
* Run `uniffi-bindgen scaffolding ./src/lib.idl` to generate a bunch of boilerplate rust code that exposes this API as a C-compatible FFI layer.
  * Update `./src/lib.rs` to `include!()` this scaffolding code as part of your crate.
* `cargo build` your crate as normal to produce a shared library.
* Run `uniffi-bindgen generate ./src/lib.idl -l kotlin` to generate a Kotlin library that can load your shared library
  and expose it to Kotlin code using your nice high-level component API!
  * Try passing `-l swift` or `-l python` to produce bindings for other languages.

For a concrete example, check out [`./examples/arithmetic/`](./examples/arithmetic/README.md) for a small
but fully-functional crate built in this style.


## Why?

There are plenty of potential ways to solve this problem, and the one that's right for us might not
be right for you. You can read a little more about the considerations and trade-offs
that lead to the current approach in our [Architecture Decision Records](./docs/adr/README.md),
starting with [this motivational document](./docs/adr/0000-whats-the-big-idea.md).

## Status

This is all very experimental and incomplete, but we do have some basic examples working, implementing
functions and datatypes in Rust and manipulating them from Kotlin, Swift, and Python.
Take a look in the [`./examples/`](./examples/) directory to see them in action.

### Component Interface

In addition to writing your component functionality as a rust crate, you will need to provide
an abstract description of your component's API using a variant of [WebIDL](https://en.wikipedia.org/wiki/Web_IDL).

This turns out to be a bit of an awkward fit, but good enough for a first version.
In the future we may be able to generate the Interface Definition from annotations on the Rust code
(in the style of `wasm_bindgen` or perhaps the [`cxx` crate](https://github.com/dtolnay/cxx))
rather than from a separate IDL file.

The code for defining a component interface lives in [./uniffi_bindgen/src/interface/mod.rs](./uniffi_bindgen/src/interface/mod.rs)
and is currently the best source of truth for syntax and semantics of the IDL.

Things that are implemented so far:

* Primitive numeric types, equivalents to those offered by Rust (`u32`, `f64`, etc).
* Strings (which are always unicode, like Rust's `String`).
* C-style enums (just the discriminant, no associated data).
* C-style structs containing named fields (we call these *records*).
* Sequences of all of the above (like Rust's `Vec<T>`).
* Optional instances of all of the above (like Rust's `Option<T>`).
* Stand-alone functions.
* Simple stateful objects with callable methods.
* Basic error reporting.

Things that are not implemented yet:

* Enums with associated data.
* Union types.
* Efficient access to binary data (like Rust's `Vec<u8>`).
* Passing object references to functions or methods.
* Callbacks

Things that are unlikely to ever be implemented (but I guess you never know..!)

* Fancy generics.
* Fancy lifetimes.
* Closures


### Rust Scaffolding

To expose the Rust code to foreign languages, we must first "flatten" it into a C-compatible FFI layer
that can be compiled into a dynamic library, then loaded from the foreign language. We call the code
for generating this FFI layer the "scaffolding", the code for which lives in
[./uniffi_bindgen/src/scaffolding.rs](./uniffi_bindgen/src/scaffolding.rs). The current implementation
uses [`askama`](https://docs.rs/askama/) for templating because it seems to give nice integration with the Rust type system.

Currently, you need to generate the scaffolding code from the IDL file and then `include!()` it
into the `lib.rs` or your crate. You can generate it by hand using the `uniffi_bindgen` CLI, like this:

```
uniffi-bindgen scaffolding ./src/example.idl
```

Or you can use the `generate_scaffolding(idl_file: &str)` function in [./uniffi_build/src/lib.rs](./uniffi_build/src/lib.rs)
to do this automatically from your crate's `build.rs` file.

If the definitions in your IDL file do not match the Rust code you've written for your component,
you will get an error when trying to compile the crate. We've made some modest effort to ensure
that the compilation errors will help point you in the right direction, but for now, you may have
to do a little digging to figure out the source of the mis-match.

### Foreign Language Bindings

We support generating code for Kotlin and Swift, as well as some partial support for Python.
The generated code works but contains some known inefficiencies, which we'll work on over time.
The code for the different foreign languages lives under [./uniffi_bindgen/src/bindings/](./uniffi_bindgen/src/bindings/).

You can generate the bindings from the IDL by using the `uniffi_bindgen` CLI, like this:

```
uniffi-bindgen generate src/arithmetic.idl -l kotlin
```

Or `-l swift` for swift, or `-l python` for python.

It's up to you to combine the generated foreign-language code and the compiled rust crate into
a suitable format for distribution (e.g. a `.jar` or android package for Kotlin).

## Code of Conduct
Please check the project's [code of conduct](./CODE_OF_CONDUCT.md)
