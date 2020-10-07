# Foreign-language bindings

As stated in the [Overview](../Overview.md), this library and tutorial does not cover *how* to ship a Rust library on mobile, but how to generate bindings for it, so this section will only cover that.

## Kotlin

Run
```
uniffi-bindgen generate src/math.udl --language kotlin
```
then have a look at `src/uniffi/math/math.kt`

## Swift

Run
```
uniffi-bindgen generate src/math.udl --language swift
```
then check out `src/math.swift`

Note that these commands could be integrated as part of your gradle/XCode build process.

This is it, you have an MVP integration of uniffi in your project.
