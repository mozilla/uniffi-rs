# Example uniffi component: "Arithmetic"

This is a minimal example of how to write a Rust component using uniffi and consume it from Kotlin.
It doesn't exercise any tricky bits of the FFI so it's a nice place to start. We have
the following so far:

* [`./src/arithmetic.idl`](./src/arithmetic.idl), the component interface definition which exposes two
  plain functions "add" and "sub". This is processed by functions in [`./build.rs`](./build.rs)
  to generate Rust scaffolding for the component.
* [`./src/lib.rs`](./src/lib.rs), the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* [`./src/main.rs`](./src/main.rs) generates a helper executable that can be used for working with
foreign language bindings, while guaranteeing that those bindings use the same version of `uniffi`
as the compiled component.
* A tiny example kotlin script in [`./example.kts`](./kotlin.kts) that imports the component, calls
  one of its methods and prints the result.

If you want to try it out, you will need:

* The [Kotlin command-line tools](https://kotlinlang.org/docs/tutorials/command-line.html), particularly `kotlinc`.
* The [Java Native Access](https://github.com/java-native-access/jna#download) JAR downloaded and its path
  added to your `$CLASSPATH` environment variable.

With that in place, try the following:

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_arithmetic`
  in `./target/debug/`.
* Run `cargo run -- generate`. That generates component bindings for Kotlin and compiles them into
  `./target/debug/arithmetic.jar`.
* Run `cargo run -- exec example.kts`. That uses the generated bindings to do some simple arithmetic
  from Kotlin, and prints out the results.
* Run `cargo. run -- exec` to get a Kotlin shell in which you can import the bindings for yourself
  and play around with them.

There is a *lot* of build and packaging detail to figure out here, but the basics seem to work OK.