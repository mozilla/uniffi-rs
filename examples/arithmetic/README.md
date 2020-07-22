# Example uniffi component: "Arithmetic"

This is a minimal example of how to write a Rust component using uniffi and consume it from Kotlin,
Swift, and Python. It doesn't exercise any tricky bits of the FFI so it's a nice place to start.
We have the following:

* [`./src/arithmetic.idl`](./src/arithmetic.idl), the component interface definition which exposes two
  plain functions "add" and "sub". This is processed by functions in [`./build.rs`](./build.rs)
  to generate Rust scaffolding for the component.
* [`./src/lib.rs`](./src/lib.rs), the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* [`./src/main.rs`](./src/main.rs) generates a helper executable that can be used for working with
  foreign language bindings, while guaranteeing that those bindings use the same version of `uniffi`
  as the compiled component.
* Some small test scripts that double as API examples in each target foreign language:
  * Kotlin [`./tests/bindings/test_arithmetic.kts`](./tests/bindings/test_arithmetic.kts)
  * Swift [`./tests/bindings/test_arithmetic.swift`](./tests/bindings/test_arithmetic.swift)
  * Python [`./tests/bindings/test_arithmetic.py`](./tests/bindings/test_arithmetic.py)

If you want to try it out, you will need:

* The [Kotlin command-line tools](https://kotlinlang.org/docs/tutorials/command-line.html), particularly `kotlinc`.
* The [Java Native Access](https://github.com/java-native-access/jna#download) JAR downloaded and its path
  added to your `$CLASSPATH` environment variable.
* Python 3
* The [Swift command-line tools](https://swift.org/download/), particularly `swift`, `swiftc` and
  the `Foundation` package.

With that in place, try the following:

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_arithmetic`
  in `../../target/debug/`.
* Run `cargo run -- generate`. That generates component bindings for each target language:
    * `../../target/debug/arithmetic.kt` for Kotlin
    * `../../target/debug/arithmetic.swift` for Swift
    * `../../target/debug/arithmetic.py` for Python
* Run `cargo test`. This exercises the foreign language bindings via the scripts in `./tests/bindings/`.
* Run `cargo run -- exec tests/bindings/test_arithmetic.kts`. This will directly execute the Kotlin
  test script. Try the same for the other languages.
* Run `cargo. run -- exec -l python` to get a Python shell in which you can import the generated
  module for yourself and play around with it. Try `import arithmetic` and go from there!
