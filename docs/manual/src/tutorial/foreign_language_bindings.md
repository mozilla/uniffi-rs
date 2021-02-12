# Foreign-language bindings

As stated in the [Overview](../Overview.md), this library and tutorial does not cover *how* to ship a Rust library on mobile, but how to generate bindings for it, so this section will only cover that.

## Kotlin

Run
```
uniffi-bindgen generate src/lib.rs --language kotlin
```
then have a look at `src/uniffi/math/math.kt`

## Swift

Run
```
uniffi-bindgen generate src/lib.rs --language swift
```
then check out `src/math.swift`

Note that these commands could be integrated as part of your gradle/XCode build process.

This is it, you have an MVP integration of UniFFI in your project!

NOTE: In future, we may support doing the bindings generation from a `cargo` subcommand,
like so:

```
cargo uniffi generate --language kotlin
```

The advantage of this approach would be that you don't have to specify the path
to a particular Rust file, the tool just figures it out by looking at the current
crate.
