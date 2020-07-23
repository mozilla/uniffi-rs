# Example uniffi component: "Todolist"

This example is designed to demonstrate a simple interaction using strings.

* [`./src/todolist.idl`](./src/todolist.idl), the component interface definition which defines the main object and its methods. This is processed by functions in [`./build.rs`](./build.rs)
  to generate Rust scaffolding for the component.
* [`./src/lib.rs`](./src/lib.rs), the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* [`./src/main.rs`](./src/main.rs) generates a helper executable that can be used for working with
  foreign language bindings, while guaranteeing that those bindings use the same version of `uniffi`
  as the compiled component.
* Some small test scripts that double as API examples in each target foreign language:
  * Kotlin [`./tests/bindings/test_todolist.kts`](./tests/bindings/test_todolist.kts)
  * Swift [`./tests/bindings/test_todolist.swift`](./tests/bindings/test_todolist.swift)
  * Python [`./tests/bindings/test_todolist.py`](./tests/bindings/test_todolist.py)

If you want to try it out, you will need:

* The [Kotlin command-line tools](https://kotlinlang.org/docs/tutorials/command-line.html), particularly `kotlinc`.
* The [Java Native Access](https://github.com/java-native-access/jna#download) JAR downloaded and its path
  added to your `$CLASSPATH` environment variable.
* Python 3
* The [Swift command-line tools](https://swift.org/download/), particularly `swift`, `swiftc` and
  the `Foundation` package.

With that in place, try the following:

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_todolist`
  in `../../target/debug/`.
* Run `cargo run -- generate`. That generates component bindings for each target language:
    * `../../target/debug/todolist.kt` for Kotlin
    * `../../target/debug/todolist.swift` for Swift
    * `../../target/debug/todolist.py` for Python
* Run `cargo test`. This exercises the foreign language bindings via the scripts in `./tests/bindings/`.
* Run `cargo run -- exec tests/bindings/test_todolist.kts`. This will directly execute the Kotlin
  test script. Try the same for the other languages.
* Run `cargo. run -- exec -l python` to get a Python shell in which you can import the generated
  module for yourself and play around with it. Try `import todolist` and go from there!
