# Enumerations

The interface definition can include Rust enums using standard syntax.
UniFFI supports both flat C-style enums:

```rust
enum Animal {
    Dog,
    Cat,
}
```

As well as enums with associated data, as long as the variants have named fields:

```rust
enum IpAddr {
  V4 {q1: u8, q2: u8, q3: u8, q4: u8},
  V6 {addr: string},
}
```

These will be exposed to the foreign-language bindings using a suitable
native enum syntax. For example:

* In Kotlin, flat enums are exposed as an `enum class` while enums with
  associated data are exposed as a `sealed class`.
* In Swift, enums are exposed using the native `enum` syntax.

Like [records](./records.md), enums are only used to communicate data
and do not have any associated behaviours.

The requirement of having only named fields may be removed in future.