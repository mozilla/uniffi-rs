# Enums in UDL

[Our simple enum example](../types/enumerations.md) is defined in UDL as:

```idl
enum Animal {
  "Dog",
  "Cat",
};
```

## Enums with fields

Enumerations with associated data require a different syntax,
due to the limitations of using WebIDL as the basis for UniFFI's interface language.
An enum like `IpAddr` is specifiedl in UDL like:

```idl
[Enum]
interface IpAddr {
  V4(u8 q1, u8 q2, u8 q3, u8 q4);
  V6(string addr);
};
```

These fields do not currently support default values in UDL,
but defaults are available to proc-macros.

## Remote, non-exhaustive enums

One corner case is an enum that's defined in another crate and has the [non_exhaustive` attribute](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute).

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
