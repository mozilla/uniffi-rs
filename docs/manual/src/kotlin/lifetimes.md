# Kotlin Lifetimes

All interfaces exposed via Kotlin expose a public API for freeing
the Kotlin wrapper object in lieu of reliable finalizers. This is done
by making the "base class" for all such generated objects implement the
`Disposable` and `AutoCloseable` interfaces.

As such, these wrappers all implement a `close()` method, which must be
explicitly called to ensure the associated Rust resources are reclaimed.

The best way to arrange for this to be called at the right time is beyond
the scope of this document; you should consult the official documentation for
`AutoClosable`, but one common pattern is the Kotlin
[use function](https://kotlinlang.org/api/latest/jvm/stdlib/kotlin/use.html).

## Nested objects

We also need to consider what happens when objects are contained in other objects.
The current situation is:

* Dictionaries that contain interfaces implement `AutoClosable` with their close() method closing
  any contained interfaces.

* Enums can't currently contain interfaces.

* Lists/Maps don't implement `AutoClosable`; if you have a list/map of interfaces
  you need to close each one individually.
