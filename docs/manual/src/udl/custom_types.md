# Custom types

Custom types offer a way to use user-defined types in your interface that don't derive one of the UniFFI type traits (`uniffi::Record`, `uniffi::Enum`, `uniffi::Object`, etc.).
Instead, custom types are converted to/from a existing type that does derive those traits when being passed across the FFI.

Custom types are often used with structs that wraps a primitive type, for example `Guid(String)`.
These types can be passed across the FFI as the primitive type, which can be more efficient than passing a struct.
The foreign bindings will treat these types as the primitive type, for example `Guid` could appear a string to the foreign code.

Custom types can also be customized on the foreign side.  For example, a URL could be:
* Represented by the `url::Url` type in Rust
* Passed across the FFI as a string
* Represented by the `java.net.URL` type in Kotlin

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

 - Values passed to the foreign code will be converted using `<SerializableStruct as TryInto<String>>` then lowered as a `String`.
     - The `TryInto::Error` type can be anything that implements `Into<anyhow::Error>`.
     - If `String` implements `TryFrom<SerializableStruct>` this will also work, since there's a blanket impl in the core library. 
 - Values passed to the Rust code will lifted as a `String` then converted using `<String as Into<SerializableStruct>>`.
     - If `SerializableStruct` implements `From<String>` this will also work, since there's a blanket impl in the core library. 

### custom_type! with manual conversions

You can also manually specify the conversions by passing an extra param to the macro:

```rust
uniffi::custom_type!(SerializableStruct, String, {
    into_existing: |s| s.serialize(),
    try_from_existing: |s| s.deserialize(),
});
```

### custom_newtype!

The custom_newtype! macro is able to handle Rust newtype-style structs which wrap a UniFFI type.

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
    into_existing: |r| LogRecordData {
        level: r.level(),
        message: r.to_string(),
    }
    try_from_existing: |r| LogRecord::builder()
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
    into_existing: |handle| handle.0,
    try_from_existing: |value| match value {
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
into_existing = "{}.toString()"
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
