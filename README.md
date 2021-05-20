# UniFFI - a multi-language bindings generator for rust

- [User manual](https://mozilla.github.io/uniffi-rs/)

This is a little experiment in building cross-platform components in Rust, based on things
we've learned in the [mozilla/application-services](https://github.com/mozilla/application-services)
project.

It's currently at the "minimally useful" stage - the core idea works and is being used in
a number of shipping products. However, there are still a lot of missing details and it needs a lot of polish.
We're currently focussed on removing some of the paper-cuts we feel in these shipping products, but believe
this tool has a bright future for many FFI requirements.

If this sounds interesting to you, please dive in! You can:

* File or work on [issues](https://github.com/mozilla/uniffi-rs/issues) here in GitHub.
* Join discussions in the [Cross-Platform Rust Components](https://chat.mozilla.org/#/room/#rust-components:mozilla.org)
  room on Matrix.
* Review the project's [code of conduct](./CODE_OF_CONDUCT.md).

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

Using UniFFI, you can:

* Implement your software component as a `cdylib` crate in Rust; let's say the code is in `./src/lib.rs`.
* Specify the desired component API using an *Interface Definition Language* (specifically, a variant of WebIDL) in a separate file like `./src/lib.udl`.
* Run `uniffi-bindgen scaffolding ./src/lib.udl` to generate a bunch of boilerplate rust code that exposes this API as a C-compatible FFI layer.
  * Update `./src/lib.rs` to `include!()` this scaffolding code as part of your crate.
* `cargo build` your crate as normal to produce a shared library.
* Run `uniffi-bindgen generate ./src/lib.udl -l kotlin` to generate a Kotlin library that can load your shared library
  and expose it to Kotlin code using your nice high-level component API!
  * Try passing `-l swift` or `-l python` to produce bindings for other languages.

For a concrete example, check out [`./examples/arithmetic/`](./examples/arithmetic) for a small
but fully-functional crate built in this style.


## Why?

There are plenty of potential ways to solve this problem, and the one that's right for us might not
be right for you. You can read a little more about the considerations and trade-offs
that lead to the current approach in our [Architecture Decision Records](./docs/adr/README.md),
starting with [this motivational document](./docs/adr/0000-whats-the-big-idea.md).

### Thread Safety

It is your responsibility to ensure the structs you expose via uniffi are
all `Send+Sync` - this will be enforced by the Rust compiler and in the future,
uniffi will make no attempt to help you with this.

However, early versions of this crate automatically wrapped rust structs in a
mutex, thus implicitly making the interfaces thread-safe and safe to be called
over the FFI by any thread. However, in practice we found this to be a
mis-feature, so version 0.7 first introduced the ability for the component
author to opt-out of this implicit wrapping and take care of thread-safety
themselves by adding a `[Threadsafe]` attribute to the interface.

We've recently taken this further; as of version 0.9.0, interfaces not marked
as `[Threadsafe]` will issue a deprecation warning, and support for this will
be removed entirely at some (reasonably soon) point in the future.

If you are seeing these deprecation warnings, you should upgrade your component
as soon as possible. For an example of what kind of effort is required to make
your interfaces thread-safe, you might like to see [this
commit](https://github.com/mozilla/uniffi-rs/commit/454dfff6aa560dffad980a9258853108a44d5985)
where we made one the examples thread-safe.

See also [adr-0004](https://github.com/mozilla/uniffi-rs/blob/main/docs/adr/0004-only-threadsafe-interfaces.md)
which outlines the reasoning behind this decision.

### Component Interface

In addition to writing your component functionality as a rust crate, you will need to provide
an abstract description of your component's API using a variant of [WebIDL](https://en.wikipedia.org/wiki/Web_IDL).

This turns out to be a bit of an awkward fit, but good enough for a first version.
In the future we may be able to generate the Interface Definition from annotations on the Rust code
(in the style of `wasm_bindgen` or perhaps the [`cxx` crate](https://github.com/dtolnay/cxx))
rather than from a separate UDL (UniFFI Definition Language) file.

The code for defining a component interface lives in [./uniffi_bindgen/src/interface/mod.rs](./uniffi_bindgen/src/interface/mod.rs)
and is currently the best source of truth for syntax and semantics of the UDL.

Things that are implemented so far:

* Primitive numeric types, equivalents to those offered by Rust (`u32`, `f64`, etc).
* Strings (which are always UTF-8, like Rust's `String`).
* Enums, including enums with associated data (aka "tagged unions" or "sum types").
* C-style structs containing named fields (we call these *records*).
* Sequences of all of the above (like Rust's `Vec<T>`).
* Optional instances of all of the above (like Rust's `Option<T>`).
* Stand-alone functions.
* Simple stateful objects with callable methods.
* Basic error reporting.
* Callbacks (although with limitations)

Things that are not implemented yet:

* Union types.
* Efficient access to binary data (like Rust's `Vec<u8>`).
* Passing object references to functions or methods.

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

Currently, you need to generate the scaffolding code from the UDL file and then `include!()` it
into the `lib.rs` or your crate. You can generate it by hand using the `uniffi_bindgen` CLI, like this:

```
uniffi-bindgen scaffolding ./src/example.udl
```

Or you can use the `generate_scaffolding(udl_file: &str)` function in [./uniffi_build/src/lib.rs](./uniffi_build/src/lib.rs)
to do this automatically from your crate's `build.rs` file.

If the definitions in your UDL file do not match the Rust code you've written for your component,
you will get an error when trying to compile the crate. We've made some modest effort to ensure
that the compilation errors will help point you in the right direction, but for now, you may have
to do a little digging to figure out the source of the mis-match.

### Foreign Language Bindings

We support generating code for Kotlin and Swift, as well as some partial support for Python.
The generated code works but contains some known inefficiencies, which we'll work on over time.
The code for the different foreign languages lives under [./uniffi_bindgen/src/bindings/](./uniffi_bindgen/src/bindings/).

You can generate the bindings from the UDL by using the `uniffi_bindgen` CLI, like this:

```
uniffi-bindgen generate src/arithmetic.udl -l kotlin
```

Or `-l swift` for swift, or `-l python` for python.

It's up to you to combine the generated foreign-language code and the compiled rust crate into
a suitable format for distribution (e.g. a `.jar` or android package for Kotlin).

## Code of Conduct
Please check the project's [code of conduct](./CODE_OF_CONDUCT.md).

## Release Process

We use [cargo-release](https://crates.io/crates/cargo-release) to simplify the release process.
It's not (yet) quite an ideal fit for our workflow, but it helps! Steps:

1. Start a new branch for the release. We typically use a single branch for all point releases,
   so it should be named something like `release-v0.6.x`:
    * `git checkout -b release-v{MAJOR}.{MINOR}.x`
    * `git push -u origin release-v{MAJOR}.{MINOR}.x`
2. Run `cargo release --dry-run -vv {MAJOR}.{MINOR}.{PATCH}` and check that the things
   it is proposing to do seem reasonable.
3. Run `cargo release {MAJOR}.{MINOR}.{PATCH}` to bump version numbers and
   publish the release to crates.io.
4. Run `git commit --amend` to fix up the version number in the commit message.
    * Manually replace `{{version}}` with `v{MAJOR}.{MINOR}.{PATCH}`.
    * This is a limitation of using `cargo release` in a workspace,
      ref [sunng87/cargo-release#222](https://github.com/sunng87/cargo-release/issues/222)
5. Tag the release commit in github.
    * `git tag v{MAJOR}.{MINOR}.{PATCH}`
    * `git push origin v{MAJOR}.{MINOR}.{PATCH}`
6. Push your branch, and make a PR to request it be merged to the main branch.
    * `git push origin`
