# Custom types

Custom types allow you to create a new UniFFI type which is passed over the FFI as another "bridge" type.
For example, you might have a type named `Url` which has a bridge type `String`, or a `Handle` bridged by an `i64`.

Any valid type can be a bridge type - not only builtins, but structs, records, enums etc.

This not only allows using types which otherwise would be impossible over the FFI (eg, `url::Url`), but other interesting "newtype" options to extend the type system.

The foreign bindings will treat these types as the bridge type - but they may optionally transform the type. For example, our `Url` has a bridged type of `string` - we could choose for Kotin to either get that as a `String`, or supply a conversion to/from a Kotlin `java.net.URL`.

This would mean that `Url` would be:
* Represented by the `url::Url` type in Rust
* Passed across the FFI as a string
* Represented by the `java.net.URL` type in Kotlin

For terminology, we lean on our existing [lifting and lowering](../internals/lifting_and_lowering.md); in the same way an FFI type is "lifted" into the Rust type, and a Rust type is "lowered" into to FFI, here the bridge type is lifted into our custom type, and our custom type is lowered into the bridge type.

This creates a 2-step lifting/lowering process: our custom type is lifted/lowered to/from the bridge type, then that bridge type lifted/lowered to/from the actual FFI type.

By default, we assume some `Into/From` relationships between the types, but you can also supply conversion closures.

## Custom types in the scaffolding code

### `custom_type!`

Use the `custom_type!` macro to define a new custom type.

The simplest case is for a type with `Into/From` already setup - eg, our `Handle`

```rust
/// handle which wraps an integer
pub struct Handle(i64);

// Defining `From<Handle> for i64` also gives us `Into<i64> for Handle`
impl From<Handle> for i64 {
    fn from(val: Handle) -> Self {
        val.0
    }
}

uniffi::custom_type!(Handle, i64);
```
and `Handle` can be used in Rust, while foreign bindings will use `i64` (or optionally converted, see below)

### `custom_type!` conversions

You can also manually specify the conversions by passing extra params to the macro.
Use this when the trait implementations do not exist, or they aren't desirable for some reason.

```rust
uniffi::custom_type!(SerializableStruct, String, {
    // Lowering our Rust SerializableStruct into a String.
    lower: |s| s.serialize(),
    // Lifting our foreign String into our Rust SerializableStruct
    try_lift: |s| s.deserialize(),
});
```

If you do not supply conversions to and from the bridge type, and assuming `SerializableStruct` and `String`, the following is used:

- Values lowered to the foreign code will be converted using `Into<String>` then lowered as a `String` value.
- Values lifted to the Rust code (eg, a `String`) is then converted using `<String as TryInto<SerializableStruct>>`;
the `TryInto::Error` type can be anything that implements `Into<anyhow::Error>`.
- `TryFrom<String>` and `From<SerializableStruct>` will also work, using the blanket impl from the core library.

### `custom_newtype!`

The `custom_newtype!` macro is able to handle Rust newtype-style structs which wrap a UniFFI type.

eg, our `Handle` object above could be declared as:
```rust
/// Handle which wraps an integer
pub struct Handle(i64);

/// `Handle` objects will be passed across the FFI the same way `i64` values are.
uniffi::custom_newtype!(Handle, i64);
```

### UDL

Define custom types in UDL via a `typedef` with a `Custom` attribute, specifying the UniFFI type
followed by the custom type.

```idl
[Custom]
typedef i64 Handle;
```

**note**: you must still call the `custom_type!` or `custom_newtype!` macros in your Rust code, as described above.


#### Using custom types from other crates

To use custom types from other crates from UDL, use a typedef wrapped with the `[External]` attribute.

For example, if another crate wanted to use the examples here:

```idl
[External="crate_defining_handle_name"]
typedef i64 Handle;

[External="crate_defining_log_record_name"]
typedef dictionary LogRecord;
```
## User-defined types

All examples so far in this section convert the custom type to a builtin type.
It's also possible to convert them to a user-defined type (Record, Enum, Interface, etc.).
For example you might want to convert `log::Record` class into a UniFFI record:

```rust

pub type LogRecord = log::Record;

#[derive(uniffi::Record)]
pub type LogRecordData {
    level: LogLevel,
    message: String,
}

uniffi::custom_type!(LogRecord, LogRecordData, {
    lower: |r| LogRecordData {
        level: r.level(),
        message: r.to_string(),
    }
    try_lift: |r| Ok(LogRecord::builder()
        .level(r.level)
        .args(format_args!("{}", r.message))
        .build())
});

```

## Error handling during conversion

You might have noticed that the `try_lift` function returns a `uniffi::Result<Self>` (which is an
alias for `anyhow::Result`) and might be wondering what happens if you return an `Err`.

It depends on the context. In short:

* If the value is being used as an argument to a function/constructor that does not return
  a `Result` (ie, does not have the `throws` attribute in the .udl), then the uniffi generated
  scaffolding code will `panic!()`

* If the value is being used as an argument to a function/constructor that *does* return a
  `Result` (ie, does have a `throws` attribute in the .udl), then the uniffi generated
  scaffolding code will use `anyhow::Error::downcast()` to try and convert the failure into
  that declared error type and:
  * If that conversion succeeds, it will be used as the `Err` for the function.
  * If that conversion fails, it will `panic()`

### Example
For example, consider the following UDL:
```idl
[Custom]
typedef i64 Handle;

[Error]
enum ExampleError {
    "InvalidHandle"
};

namespace errors_example {
    take_handle_1(Handle handle);

    [Throws=ExampleError]
    take_handle_2(Handle handle);
}
```

and the following Rust:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ExampleError {
    #[error("The handle is invalid")]
    InvalidHandle,
}

uniffi::custom_type!(ExampleHandle, Builtin, {
    lower: |handle| handle.0,
    try_lift: |value| match value {
        0 => Err(ExampleErrors::InvalidHandle.into()),
        -1 => Err(SomeOtherError.into()), // SomeOtherError decl. not shown.
        n => Ok(Handle(n)),
    }
})
```

The behavior of the generated scaffolding will be:

* Calling `take_handle_1` with a value of `0` or `-1` will always panic.
* Calling `take_handle_2` with a value of `0` will throw an `ExampleError` exception
* Calling `take_handle_2` with a value of `-1` will always panic.
* All other values will return `Ok(ExampleHandle)`

## Custom types in the bindings code

*Note: The facility described in this document is not yet available for the Ruby bindings.*

By default, the foreign bindings just see the builtin type - eg, the bindings will get an integer
for the `Handle`.

However, custom types can also be converted on the bindings side.  For example, a Url type could be
configured to use the `java.net.URL` class in Kotlin by adding code like this to `uniffi.toml`:

```toml
[bindings.kotlin.custom_types.Url]
# Name of the type in the Kotlin code
type_name = "URL"
# Classes that need to be imported
imports = [ "java.net.URL" ]
# Expression to convert the builtin type the custom type.  In this example, `{}` will be replaced with the int value.
lift = "URL({})"
# Expression to convert the custom type to the builtin type.  In this example, `{}` will be replaced with the URL value.
lower = "{}.toString()"
```

Here's how the configuration works in `uniffi.toml`.

* Create a `[bindings.{language}.custom_types.{CustomTypeName}]` table to enable a custom type on a bindings side.  This has several subkeys:
  * `type_name` (Optional, Typed languages only): Type/class name for the
    custom type.  Defaults to the type name used in the UDL.  Note: The UDL
    type name will still be used in generated function signatures, however it
    will be defined as a typealias to this type.
  * `lift`: Expression to convert the UDL type to the custom type.  `{}` will be replaced with the value of the UDL type.
  * `lower`: Expression to convert the custom type to the UDL type.  `{}` will be replaced with the value of the custom type.
  * `imports` (Optional) list of modules to import for your `lift`/`lower` functions.
