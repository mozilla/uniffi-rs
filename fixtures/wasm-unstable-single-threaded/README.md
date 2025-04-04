# Testing the `wasm-unstable-single-threaded` feature.

This feature changes the `Sync` + `Send` bounds on some traits, especially around `Futures` and
objects.

This test should pass by dint of compiling under both `wasm32-unknown-unknown` and any non-wasm targets.
