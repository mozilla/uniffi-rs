# Example uniffi component: "Sprites"

This is a small example of how to write a Rust component using uniffi and consume it from Kotlin.
It's designed to exercise our support for stateful objects and calling their methods.
We have the following:

* [`./src/sprites.idl`](./src/sprites.idl), the component interface definition which defines some
  basic objects and their methods. This is processed by functions in [`./build.rs`](./build.rs)
  to generate Rust scaffolding for the component.
* [`./src/lib.rs`](./src/lib.rs), the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* [`./src/main.rs`](./src/main.rs) generates a helper executable that can be used for working with
  foreign language bindings, while guaranteeing that those bindings use the same version of `uniffi`
  as the compiled component.
* A tiny example kotlin script in [`./example.kts`](./example.kts) that imports the component, calls
  some of its methods and prints the results.

If you want to try it out, you will need:

* The [Kotlin command-line tools](https://kotlinlang.org/docs/tutorials/command-line.html), particularly `kotlinc`.
* The [Java Native Access](https://github.com/java-native-access/jna#download) JAR downloaded and its path
  added to your `$CLASSPATH` environment variable.

With that in place, try the following:

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_sprites`
  in `./target/debug/`.
* Run `cargo run -- generate`. That generates component bindings for Kotlin and compiles them into
  `./target/debug/sprites.jar`.
* Run `cargo run -- exec example.kts`. That uses the generated bindings to do some simple spite manipulation
  from Kotlin, and prints out the results.
* Run `cargo. run -- exec` to get a Kotlin shell in which you can import the bindings for yourself
  and play around with them.

There is a *lot* of build and packaging detail to figure out here, but the basics seem to work OK.
