# Example uniffi components

This directory contains some small example components implemented using uniffi. It's currently being used
more as a living test environment than user-facing docs, but hopefully it gives you a bit of an idea of
what we're up to with this crate.

Newcomers are recommended to explore them in the following order:

* [`./arithmetic/`](./arithmetic/) is the most minimal example - just some plain functions that operate
  on integers, and a simple enum.
* [`./geometry/`](./geometry/) shows how to use records and nullable types for working with more complex
  data.
* [`./sprites/`](./sprites/) shows how to work with stateful objects that have methods, in classical
  object-oriented style.
* [`./todolist`](./todolist/) is a simple todolist that only adds items and shows the last item, meant to show how interacting with strings works.
* [`.rondpoint`](./rondpoint/) exercises complex data types through roundtripping.
* [`./fxa-client`](./fxa-client/) doesn't work yet, but it contains aspirational example of what the IDL
  might look like for an actual real-world component.

Each example has the following structure:

* `src/<namespace>.idl`, the component interface definition which defines the main object and its methods. This is processed by functions in `build.rs`
  to generate Rust scaffolding for the component.
* `src/lib.rs`, the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* `src/main.rs` generates a helper executable that can be used for working with
  foreign language bindings, while guaranteeing that those bindings use the same version of `uniffi`
  as the compiled component.
* Some small test scripts that double as API examples in each target foreign language:
  * Kotlin `tests/bindings/test_<namespace>.kts`
  * Swift `tests/bindings/test_<namespace>.swift`
  * Python `tests/bindings/test_<namespace>.py`

If you want to try them out, you will need:

* The [Kotlin command-line tools](https://kotlinlang.org/docs/tutorials/command-line.html), particularly `kotlinc`.
* The [Java Native Access](https://github.com/java-native-access/jna#download) JAR downloaded and its path
  added to your `$CLASSPATH` environment variable.
* Python 3
* The [Swift command-line tools](https://swift.org/download/), particularly `swift`, `swiftc` and
  the `Foundation` package.

With that in place, try the following:

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_<namespace>`
  in `target/debug/`.
* Run `cargo run -- generate`. That generates component bindings for each target language:
    * `target/debug/uniffi/<namespace>/<namespace>.kt` for Kotlin
    * `target/debug/<namespace>.swift` for Swift
    * `target/debug/<namespace>.py` for Python
* Run `cargo test`. This exercises the foreign language bindings via the scripts in `tests/bindings/`.
* Run `cargo run -- exec tests/bindings/test_<namespace>.kts`. This will directly execute the Kotlin
  test script. Try the same for the other languages.
* Run `cargo run -- exec -l python` to get a Python shell in which you can import the generated
  module for yourself and play around with it. Try `import <namespace>` and go from there!
