
## What?

We're interested in building re-useable components for our browsers.
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
* Optionally, describe parts of your component API using proc-macros directly in `lib.rs`.
* Optionally, describe parts of your component API using an *Interface Definition Language* in a separate file like `./src/lib.udl`. UniFFI will generate a bunch of boilerplate Rust code that exposes this API as a C-compatible FFI layer, and include it as part of your crate.
* `cargo build` your crate as normal to produce a shared library.
* Run `uniffi-bindgen generate ... -l kotlin` (see [the bindgen docs](./tutorial/foreign_language_bindings.md) for omitted arg details)
  to generate a Kotlin library that can load your shared library
  and expose it to Kotlin code using your nice high-level component API!
  * Or `-l swift` or `-l python` to produce bindings for other languages.

## Why?

There are plenty of potential ways to solve this problem, and the one that's right for us might not
be right for you. You can read a little more about the considerations and trade-offs
that lead to the current approach in our [Architecture Decision Records](https://github.com/mozilla/uniffi-rs/tree/main/docs/adr/README.md),
starting with [this motivational document](https://github.com/mozilla/uniffi-rs/tree/main/docs/adr/0000-whats-the-big-idea.md).

## Why Not?

We hope UniFFI will be useful to you! But if you're considering it for your project then
here are some tradeoffs you should keep in mind:

* UniFFI makes it easy to produce "good enough" bindings into Rust from several different target languages.
  If you want to call Rust code from just one foreign language, there may be a language-specific
  bindings tool that provides more features and better performance.
* The project is evolving fast, because it's being developed concurrently with its
  primary consumers. You should be prepared for frequent releases and non-trivial API churn
  (but we'll use semantic versioning to signal breaking changes).
* UniFFI doesn't provide an end-to-end packaging solution for your Rust code and its generated bindings.
  For example, it'll help you generate Kotlin bindings to call into Rust, but it won't help you with
  compiling the Rust code to run on Android or with packaging the bindings into an `.aar` bundle
  (but it might be useful as a building-block for such a tool!).

We also have a list of [design principles](./internals/design_principles.md) that might help
you decide whether this project is a good fit for your needs.