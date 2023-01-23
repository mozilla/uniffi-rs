# A basic test for uniffi components

This test covers async functions and methods. It also provides examples.

## Run the tests

Simply use `cargo`:

```sh
$ cargo test
```

It is possible to filter by test names, like `cargo test -- swift` to only run
Swift's tests.

## Run the examples

At the time of writing, each `examples/*` directory has a `Makefile`. They are
mostly designed for Unix-ish systems, sorry for that.

To run the examples, first `uniffi` must be compiled:

```sh
$ cargo build --release -p uniffi`
```

Then, each `Makefile` has 2 targets: `build` and `run`:

```sh
$ # Build the examples.
$ make build
$
$ # Run the example.
$ make run
```

One note for `examples/kotlin/`, some JAR files must be present, so please
run `make install-jar` first: It will just download the appropriated JAR files
directly inside the directory from Maven.