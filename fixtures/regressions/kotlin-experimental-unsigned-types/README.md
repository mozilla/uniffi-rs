# Regression test for propagating annotations on Kotlin unsigned types.

Kotlin's support for unsigned integers is currently experimental (kotlin stabilizes this api in [1.5](#kotlin-1.5)),
and requires explicit opt-in from the consuming application.
Since we can generate Kotlin code that uses unsigned types, we
need to ensure that such code is annotated with
[`@ExperimentalUnsignedTypes`][https://kotlinlang.org/api/latest/jvm/stdlib/kotlin/-experimental-unsigned-types/]
in order to avoid compilation warnings.

This crate exposes an API that uses unsigned types in a variety of
ways, and tests that the resulting Kotlin bindings can be compiled
without warnings.

---

## Kotlin 1.5

As of Kotlin 1.5, [unsigned types are stablized](https://kotlinlang.org/docs/whatsnew15.html#stable-unsigned-integer-types). This will elimate the need for this test fixture as well as any places in the code the `@ExperimentalUnsignedTypes` annotation lives. This will most likely be done when the minimum supported version of kotlin in uniffi-rs becomes 1.5.
