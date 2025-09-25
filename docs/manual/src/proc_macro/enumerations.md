## The `uniffi::Enum` derive

The `Enum` derive macro works much like the [`Record`](./records.md) derive macro. Any fields inside variants must
be named. All types that are supported as parameter and return types by `#[uniffi::export]` are
also supported as field types.

It is permitted to use this macro on a type that is also defined in the UDL file as long as the
two definitions are equal in the names and ordering of variants and variant fields, and any field
types inside variants are UniFFI builtin types; user-defined types might be allowed in the future.

```rust
#[derive(uniffi::Enum)]
pub enum MyEnum {
    Fieldless,
    WithFields {
        foo: u8,
        #[uniffi(default)]
        bar: Vec<i32>,
    },
    WithValue = 3,
}
```

Named fields within a variant can have [default values](../types/defaults.md)

### Variant Discriminants

Variant discriminants are accepted by the macro but how they are used depends on the bindings.

For example this enum:

```rust
#[derive(uniffi::Enum)]
pub enum MyEnum {
    Foo = 3,
    Bar = 4,
}
```

would give you in Kotlin & Swift:

```swift
// kotlin
enum class MyEnum {
    FOO,
    BAR;
    companion object
}
// swift
public enum MyEnum {
    case foo
    case bar
}
```

which means you cannot use the platforms helpful methods like `value` or `rawValue` to get the underlying discriminants. Adding a `repr` will allow the type to be defined in the foreign bindings.

For example:

```rust
// added the repr(u8), also u16 -> u64 supported
#[repr(u8)]
#[derive(uniffi::Enum)]
pub enum MyEnum {
    Foo = 3,
    Bar = 4,
}
```

will now generate:

```swift
// kotlin
enum class MyEnum(val value: UByte) {
    FOO(3u),
    BAR(4u);
    companion object
}

// swift
public enum MyEnum : UInt8 {
    case foo = 3
    case bar = 4
}
```

## Renaming enums

Enums can be renamed in foreign language bindings using the `name` parameter:

```rust
#[derive(uniffi::Enum)]
#[uniffi(name = "RenamedEnum")]
pub enum MyEnum {
    // ...
}
```

See [Renaming](./renaming.md) for more details on renaming functionality.
