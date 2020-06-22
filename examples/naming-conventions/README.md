# Example uniffi component: "Naming Conventions"

This is a small example of how to write a Rust component using uniffi and consume it from Kotlin.

It's designed to exercise our support for name mangling into the conventions expected by the source language and coerced in to the conventions for rust.

e.g. 

calling `getValue()` from kotlin, should call a method called `get_value()` in rust.

We have the following:

* [`./src/naming-conventions.idl`](./src/naming-conventions.idl), the component interface definition which defines some
  some toy objects and methods with names that don't follow naming conventions for one of the python, rust or kotlin. This is processed by functions in
  [`./build.rs`](./build.rs) to generate Rust scaffolding for the component.
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

* Run `cargo build`. That compiles the component implementation into a native library named `uniffi_geometry`
  in `./target/debug/`.
* Run `cargo run -- generate`. That generates component bindings for Kotlin and compiles them into
  `./target/debug/geometry.jar`.
* Run `cargo run -- exec example.kts`. That uses the generated bindings to do show rountripping
  from Kotlin, and prints out the results.
* Run `cargo run -- exec example.py`. That uses the generated bindings to do show rountripping
from Python, and prints out the results.
* Run `cargo. run -- exec` to get a Kotlin shell in which you can import the bindings for yourself
  and play around with them.

There is a *lot* of build and packaging detail to figure out here, but the basics seem to work OK.
