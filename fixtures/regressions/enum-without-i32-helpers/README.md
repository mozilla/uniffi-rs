# Regression test for issue #356

In issue [#356](https://github.com/mozilla/uniffi-rs/issues/356)
we discovered that the generated bindings for Enum classes were
using helper functions from the i32 data type, but we would only
generate those functions if the component interface was also
using that type directly.

This crate is a minimal reproduction of the issue, and its Kotlin
bindings would fail to compile in the presence of the bug.
