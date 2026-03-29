`uniffi-bindgen-kotlin-jni` is an experimental bindgen system that generates
JNI-based Rust scaffolding and Kotlin bindings to call into it.

# Code generation

`uniffi-bindgen-kotlin-jni` uses `uniffi_parse_rs` to parse the Rust source
and create the metadata needed to generate the bindings/scaffolding.

The scaffolding is generated using the `uniffi_pipeline` framework and Askama templates
rather than from the macros.

Scaffolding is generated for a single crate only.
The main reason is that we only want to parse the source code once.

# FFI calling convention

## Primitive arguments / return values

If possible, we try to pass arguments directly using JNI.

The following types are passed as primitives:
  * Integers, floats, and bool.  Unsigned ints are converted to their signed counterparts.
  * `String` is passed as a `jstring`
  * `Vec`, `HashMap` and `HashSet` get passed as NIO byte buffer except for some special cases:
      * `Vec<T>` where T is a primitive type gets passed as an array.
  * Objects are passed as an `i64`
  * `Option<T>` where T is an integer type with 32-bit width or less
    We can use the niche optimization on these by casting them to an `i64`
    and using `i64::MAX` for `None`.
  * `Option<T>` where T is a floating point type.
    For these, we pick a special NaN value to use for `None`
  * `Option<String>` and `Option<Vec<u8>>` also uses the niche optimization
    with `null` used for `None`.
  * Flat enums are passed using their discriminants.

# Kotlin `uniffi` package

This is a generated package that contains all the FFI functions.
Putting these functions here has some advantages over our current system:

* **Less namespace pollution**.
  With `uniffi-bindgen-kotlin` have to add public functions for UniFFI-internal operations.
  These functions need to be public to make external types work.
  However, it feels slightly wrong for them to end up in the consumer-facing package.
  Putting these functions in the `uniffi` package feels better.
* **Less duplication**.
  If multiple crates use `Vec<u8>` in their interfaces, we'll only need to define 1 read and 1 write function.
* **Simplified external types**.
  We don't need to map namespaces to package names to find the right function.

# JNI

This crate uses the low-level `jni_sys` crate rather than the high-level `jni` crate.
This allows for more control and better performance in the JNI code.
An earlier version used `jni`, but benchmarks showed something like ~2-4x increase when switching to `jni_sys`.
`jni_sys` is not much harder for us to use, since we're operating at a very low level.

`jni_sys` is also a lighter-weight dependency,
which is important since we're forcing all consumer crates to depend on it.
