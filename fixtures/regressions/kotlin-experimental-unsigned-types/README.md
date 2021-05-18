# Regression test for propagating annotations on Kotlin unsigned types.

Kotlin's support for unsigned integers is currently experimental,
and requires explicitly opt-in from the consuming application.
Since we can generate Kotlin code that uses unsigned types, we
need to ensure that such code is annotated with
[`@ExperimentalUnsignedTypes`][https://kotlinlang.org/api/latest/jvm/stdlib/kotlin/-experimental-unsigned-types/]
in order to avoid compilation warnings.

This crate exposes an API that uses unsigned types in a variety of
ways, and tests that the resulting Kotlin bindings can be compiled
without warnings.
