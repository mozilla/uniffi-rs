# Custom types

Custom types allow you to extend the UniFFI type system by converting to and from some other UniFFI
type to move data across the FFI.

## Custom types in the scaffolding code

### custom_type!

Use the `custom_type!` macro to define a new custom type.

```rust

// Some complex struct that can be serialized/deserialized to a string.
use some_mod::SerializableStruct;

/// `SerializableStruct` objects will be passed across the FFI the same way `String` values are.
uniffi::custom_type!(SerializableStruct, String);
```

By default:

 - Values passed to the foreign code will be converted using `<SerializableStruct as Into<String>>` before being lowered as a `String`.
 - Values passed to the Rust code will be converted using `<String as TryInto<SerializableStruct>>` after lifted as a `String`.
 - The `TryInto::Error` type can be anything that implements `Into<anyhow::Error>`.
 - `<String as Into<SerializableStruct>>` will also work, since there is a blanket impl in the core libary.

### custom_type! with manual conversions

You can also manually specify the conversions by passing an extra param to the macro:

```rust
uniffi::custom_type!(SerializableStruct, String, {
    from_custom: |s| s.serialize(),
    try_into_custom: |s| s.deserialize(),
});
```

### custom_newtype!

The `custom_newtype!` can trivially handle newtypes that wrap a UniFFI type.

```rust
/// handle which wraps an integer
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

**note**: UDL users still need to call the `custom_type!` or `custom_newtype!` macro in their Rust
code.

## User-defined types

All examples so far in this section convert the custom type to a builtin type.
It's also possible to convert them to a user-defined type (Record, Enum, interface, etc.).
For example you might want to convert `log::Record` class into a UniFFI record:

```rust

pub type LogRecord = log::Record;

#[derive(uniffi::Record)]
pub type LogRecordData {
    level: LogLevel,
    message: String,
}

uniffi::custom_type!(LogRecord, LogRecordData, {
    from_custom: |r| LogRecordData {
        level: r.level(),
        message: r.to_string(),
    }
    try_into_custom: |r| LogRecord::builder()
        .level(r.level)
        .args(format_args!("{}", r.message))
        .build()
});

```

## Error handling during conversion

You might have noticed that the `into_custom` function returns a `uniffi::Result<Self>` (which is an
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
    from_custom: |handle| handle.0,
    try_into_custom: |value| match value {
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
into_custom = "URL({})"
# Expression to convert the custom type to the builtin type.  In this example, `{}` will be replaced with the URL value.
from_custom = "{}.toString()"
```

Here's how the configuration works in `uniffi.toml`.

* Create a `[bindings.{language}.custom_types.{CustomTypeName}]` table to enable a custom type on a bindings side.  This has several subkeys:
  * `type_name` (Optional, Typed languages only): Type/class name for the
    custom type.  Defaults to the type name used in the UDL.  Note: The UDL
    type name will still be used in generated function signatures, however it
    will be defined as a typealias to this type.
  * `into_custom`: Expression to convert the UDL type to the custom type.  `{}` will be replaced with the value of the UDL type.
  * `from_custom`: Expression to convert the custom type to the UDL type.  `{}` will be replaced with the value of the custom type.
  * `imports` (Optional) list of modules to import for your `into_custom`/`from_custom` functions.

## Using custom types from other crates

To use custom types from other crates, use a typedef wrapped with the `[External]` attribute.
For example, if another crate wanted to use the examples above:

```idl
[External="crate_defining_handle_name"]
typedef i64 Handle;

[External="crate_defining_log_record_name"]
typedef dictionary LogRecord;
```

## Remote custom types

Custom types that convert [Remote types](./remote_ext_types.md#remote-types) defined in other crates require special handling.

1) Specify `remote` param in the `custom_type!` macro:

```rust

uniffi::custom_type!(SerializableStructFromOtherCrate, String, {
    remote,
    from_custom: |s| s.serialize(),
    try_into_custom: |s| s.deserialize(),
});
```

2) To share the custom type implementation with other crates, use the [remote_type! macro](./remote_ext_types.md#external+remote-types).
