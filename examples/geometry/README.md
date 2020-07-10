# Example uniffi component: "Geometry"

This is a small example of how to write a Rust component using uniffi and consume it from  Kotlin,
Swift, and Python. It's designed to exercise our support for structured data types; for example it
defines `Point` and `Line` records and some operations on them that might return nullable data.
We have the following:

* [`./src/geometry.idl`](./src/geometry.idl), the component interface definition which defines some
  basic geometric objects and functions that act upon them. This is processed by functions in
  [`./build.rs`](./build.rs) to generate Rust scaffolding for the component.
* [`./src/lib.rs`](./src/lib.rs), the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* [`./src/main.rs`](./src/main.rs) generates a helper executable that can be used for working with
  foreign language bindings, while guaranteeing that those bindings use the same version of `uniffi`
  as the compiled component.
* Some small test scripts that double as API examples in each target foreign language:
  * Kotlin [`./tests/bindings/test_geometry.kts`](./tests/bindings/test_geometry.kts)
  * Swift [`./tests/bindings/test_geometry.swift`](./tests/bindings/test_geometry.swift)
  * Python [`./tests/bindings/test_geometry.py`](./tests/bindings/test_geometry.py)

If you want to try it out, you will need:

* The [Kotlin command-line tools](https://kotlinlang.org/docs/tutorials/command-line.html), particularly `kotlinc`.
* The [Java Native Access](https://github.com/java-native-access/jna#download) JAR downloaded and its path
  added to your `$CLASSPATH` environment variable.
* Python 3
* The [Swift command-line tools](https://swift.org/download/), particularly `swift`, `swiftc` and
  the `Foundation` package.

With that in place, try the following:

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_geometry`
  in `../../target/debug/`.
* Run `cargo run -- generate`. That generates component bindings for each target language:
    * `../../target/debug/geometry.kt` for Kotlin
    * `../../target/debug/geometry.swift` for Swift
    * `../../target/debug/geometry.py` for Python
* Run `cargo test`. This exercises the foreign language bindings via the scripts in `./tests/bindings/`.
* Run `cargo run -- exec tests/bindings/test_geometry.kts`. This will directly execute the Kotlin
  test script. Try the same for the other languages.
* Run `cargo. run -- exec -l python` to get a Python shell in which you can import the generated
  module for yourself and play around with it. Try `import geometry` and go from there!