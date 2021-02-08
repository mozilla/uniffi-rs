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
