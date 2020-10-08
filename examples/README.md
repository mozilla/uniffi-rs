# Example uniffi components

This directory contains some small example components implemented using uniffi. It's currently being used
more as a living test environment than user-facing docs, but hopefully it gives you a bit of an idea of
what we're up to with this crate.

Newcomers are recommended to explore them in the following order:

* [`./arithmetic/`](./arithmetic/) is the most minimal example - just some plain functions that operate
  on integers, and some minimal error handling.
* [`./geometry/`](./geometry/) shows how to use records and nullable types for working with more complex
  data.
* [`./sprites/`](./sprites/) shows how to work with stateful objects that have methods, in classical
  object-oriented style.
* [`./todolist`](./todolist/) is a simplistic todo-list that can only add items and show the last item,
  meant to show how interacting with strings works.
* [`.rondpoint`](./rondpoint/) exercises complex data types by round-tripping them from the foreign-language
  code, through rust and back agian.
* [`./fxa-client`](./fxa-client/) doesn't work yet, but it contains aspirational example of what the UDL
  might look like for an actual real-world component.

Each example has the following structure:

* `src/<namespace>.udl`, the component interface definition which defines the main object and its methods.
  This is processed by functions in `build.rs` to generate Rust scaffolding for the component.
* `src/lib.rs`, the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* `Cargo.toml` configures the crate to build a `cdylib` with an appropriate name, matching the
  namespace defined in the UDL file.
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

We publish a [docker image](https://hub.docker.com/r/rfkelly/uniffi-ci) that has all of this dependencies
pre-installed, if you want to get up and running quickly.

With that in place, try the following:

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_<namespace>`
  in `../target/debug/`.
* Run `cargo test`.  This will run each of the foreign-language testcases against the compiled Rust code,
  confirming whether everything is working as intended.
* Explore the build process in more detail:
  * Run `cargo run -p uniffi_bindgen scaffolding ./src/<namespace>.udl`.
    This will generate the Rust scaffolding code which exposes a C FFI for the component.
    You can view the generatd code in `./src/<namespace>.uniffi.rs`.
  * Run `cargo run -p uniffi_bindgen generate --language kotlin ./src/arithmetic.udl`.
    This will generate the foreign-language bindings for Kotlin, which load the compiled Rust code
    and use the C FFI generated above to interact with it.
    You can view the generated code in `./src/uniffi/<namespace>/<namespace>.kt`.
  * Try using `--language swift` or `--language python` to explore the foreign-language bindings
    generated for other languages.
