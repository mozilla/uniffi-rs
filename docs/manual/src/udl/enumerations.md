# Enumerations

An enumeration defined in Rust code as

```rust
enum Animal {
    Dog,
    Cat,
}
```

Can be exposed in the UDL file with:

```idl
enum Animal {
  "Dog",
  "Cat",
};
```

## Enums with fields

Enumerations with associated data require a different syntax,
due to the limitations of using WebIDL as the basis for UniFFI's interface language.
An enum like this in Rust:

```rust
enum IpAddr {
  V4 {q1: u8, q2: u8, q3: u8, q4: u8},
  V6 {addr: string},
}
```

Can be exposed in the UDL file with:

```idl
[Enum]
interface IpAddr {
  V4(u8 q1, u8 q2, u8 q3, u8 q4);
  V6(string addr);
};
```

Only enums with named fields are supported by this syntax.

## Remote, non-exhaustive enums

One corner case is an enum that's:
  - Defined in another crate.
  - Has the [non_exhaustive` attribute](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute).

In this case, UniFFI needs to generate a default arm when matching against the enum variants, or else a compile error will be generated.
Use the `[NonExhaustive]` attribute to handle this case:

```idl
[Enum]
[NonExhaustive]
interface Message {
  Send(u32 from, u32 to, string contents);
  Quit();
};
```

**Note:** since UniFFI generates a default arm, if you leave out a variant, or if the upstream crate adds a new variant, this won't be caught at compile time.
Any attempt to pass that variant across the FFI will result in a panic.
