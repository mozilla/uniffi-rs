# Enumerations

Enums must be exposed via [UDL](../udl/enumerations.md) or [proc-macros](../proc_macro/enumerations.md)

Simple enums just work:

```rust
enum Animal {
    Dog,
    Cat,
}
```

and naturally used in the bindings:
```kotlin
Animal.Dog // kotlin
```
```swift
.dog // swift
```
```python
Animal.DOG() # python
a.is_DOG()
```

## Enums with fields

Enumerations with associated data are supported.

```rust
enum IpAddr {
  V4 {q1: u8, q2: u8, q3: u8, q4: u8},
  V6 {addr: string},
}
```

Enums can be very flexible (although [UDL](../udl/enumerations.md) doesn't support all of this)

```rust
#[derive(uniffi::Enum)]
pub enum MyEnum {
    None,
    Str(String),
    All { s: String, i: i64 }
}
```

## Remote, non-exhaustive enums

There are some [special considerations here when using UDL](../udl/enumerations.md#remote-non-exhaustive-enums)

## Exposing methods from standard Rust traits

While less useful for Enums, there are a number of standard Rust traits (`Debug`, `Eq` etc) you can expose, so, eg, Python
might generate `__repr__()` or `__eq__()` methods - [see the docs for this feature](./uniffi_traits.md).
