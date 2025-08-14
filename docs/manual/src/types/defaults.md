# Default values

You can specify a default value to be used for function arguments, and on fields in `Record`s and `Enum`s.
Exactly how they are specified depends on the context, but all use the same mechanism described here.

You can optionally specify a literal value. If not specified the "natural" default for the type will be used.

For example, an `Enum` might specify in Rust:

```rust
#[derive(uniffi::Enum)]
pub enum MyEnum {
    MyVariant {
        #[uniffi(default)]
        d: u8,
        #[uniffi(default = 1)]
        e: u8,
    },
}
```

`d` has no literal, so wll use `0`, while `e` specifies a literal.

These args can be omitted when creating the `Enum` in the foreign bindings. `Record` fields use the same mechanism.

Similarly for, eg, methods:
```rust
    #[uniffi::method(default(name = "default", size))]
    fn my_method(&self, name: String, size: u32) { ... }
```

and the foreign bindings will not need to specify those arguments.

UDL supports default values (eg, [`Record`s](../udl/records.md#default-values-for-fields), [functions](../udl/functions.md#default-values)) and that literal values are required - there's no support for a type's "natural" default.

## Natural Default

The types you can use a natural default with are below.

| Type | Default value |
|---------|----------------------|
| `Option` | `None` |
| `String` | Empty string |
| `Vec<T>` | Empty list |
| `HashMap<T>` | Empty map |
| Builtin numeric types | 0 |
| `bool` | false |
| Records | Record with all fields set to their default value (only valid if they all have defaults) |
| Objects | Primary constructor called with 0 arguments |
| Custom Types | The default value of the "bridge" type |

## Literal values

In UDL, default values are specified in-line when describing a type.
For example, a function might declare an optional arg as `void hello(optional string name = "world");`. Enum variant fields can't be expressed in UDL.

In proc-macros, default values are typically described like our example above, or slightly differently for function args. The proc-macros support the literals we describe here.

Most of the [builtin types](./builtin_types.md) support obvious values:

| Rust type            | Supported defaults
|----------------------|-------------------
| `bool`               | `true` or `false`
| `u8/i8..u64/i64`     | See below
| `f32`/`f64`          | A numeric value with optional decimal - eg, `0.1` or `0`.
| `String`             | A "quoted literal"
| `Option<T>`          | `null` in UDL, `None` in proc-macros
| `Vec<>`              | `[]`.
| `HashMap`            | `{}` in UDL only.

Custom types support the literal of their "bridge" type.

Note that `HashMap` and other types do support "natural defaults" described above.

## Integer literals

UDL has a sophisticated scheme which carries the nominal representation and radix etc.
In turn, bindings can mimic this closely, generating hex or octal literals which best
represent the value. Much of this comes "for free" from weedle.

For example, `0`, `0x0` and `00` are all supported and would be reflected
in the bindings as decimal, hex or octal numbers accordingly.

Proc macros however have only simple integer literal support - all literals will
be represented as signed or unsigned integers with radix 10 - so only decimal numbers
are supported as literals in the Rust or UDL code, and the bindings will
treat them as "normal" decimal numbers. There's no reason the proc-macros couldn't learn to do more here.

For example, `123` and `-1` are valid integer literals in proc-macros.
