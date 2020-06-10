# Example uniffi component: "Arithmetic"

This is a minimal (and very work-in-progress!) example of how to write a Rust component using
uniffi. It doesn't exercise any tricky bits of the FFI so it's a nice place to start. We have
the following so far:

* [`./arithmetic.idl`](./arithmetic.idl), the component interface definition which exposes two
  plain functions "add" and "sub". This is processed by functions in [`./build.rs`](./build.rs)
  to generate Rust scaffolding for the component, and some Kotlin bindings.
* [`./src/lib.rs`](./src/lib.rs), the core implementation of the component in Rust. This basically
  pulls in the generated Rust scaffolding via `include!()` and fills in function implementations.
* A tiny example program in [`./main.kt`](./main.kt) that imports the component in Kotlin, calls
  one of its methods and prints the result.
* Some extremely hacky code in [`./build.rs`](./build.rs) that only works on my machine (since it
  has some hard-coded file paths) that generates Kotlin bindings from the IDL, compiles them together
  with `./main.kt`, and produces a runnabe `.jar` file to exercise the component.

There is a *lot* of build and packaging detail to figure out here, but I'm able to do the following
and actually use the Rust component from Kotlin:

* Install the kotlin command-line compiler.
* Edit `build.rs` to point it to a local copy of the JNA jar.
* Run `cargo build` in this directory; observe that it creates a file `./arithmetic.jar`.
* Try to run `./arithmetic.jar` directly using `java -jar arithmetic.jar`; observe that it fails because it can't find JNA in the classpath, and I can't figure out the right command-line flags to get it to do so.
* Unpack the jar to try running it by hand:
    * `mkdir unpacked; cd unpacked`
    * `unzip ../arithmetic.jar`
    * `unzip -o /path/to/jna-5.2.0.jar`
    * `cp ../target/debug/libuniffi_example_arithmetic.dylib ./`
    * `java MainKt`
    * Observe that it correctly prints the result of some simple arithmetic!

That obviously needs to be smoother, but you get the idea :-)