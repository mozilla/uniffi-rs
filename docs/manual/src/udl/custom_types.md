# Custom types

Custom types allow users to extend the UniFFI type system to support types that normally could not be used in an interface.

As described in the [Lifting and Lowering](../internals/lifting_and_lowering.html) section, UniFFI passes types across the FFI by *lifting* and *lowering* them.
For example, when the foreign code wants to make a Rust call:

* The generated foreign code *lowers* all argument values to an primitive FFI type.
* The generated foreign code calls the FFI function with the lowered values
* The generated Rust code *lifts* the arguments to a Rust type
* The generated Rust code calls the exported function.
* The generated Rust code *lowers* the return value of the function and returns that back to the foreign side.
* The generated foreign code *lifts* the return value and returns that to the consumer code.

UniFFI supports many [builtin types](../udl/builtin_types.html) as well as user-defined [structs](..,udl/structs.html), [enumerations](..,udl/enumerations.html), [objects](..,udl/interfaces.html), etc.

Custom types allow library authors to extend the type system even further.
Instead of lifted from/lowered into a primitive FFI type, custom types are lifted from/lowered into a builtin or user-defined types, called a "bridge type".
This creates a 2-step lifting/lowering process: the custom type is lifted/lowered, then the result is lifted/lowered using the normal logic for the bridge type.
The foreign bindings will treat these types as the bridge type.

For example, suppose a library uses the `url::Url` type, which can not be lifted/lowered directly by UniFFI.
The library can define `url::Url` as a custom type, with `String` as the bridge type.
The generated Rust code will then lower `Url` in 2 steps:

* The `Url` value is lowered to a `String`
* The `String` value is lowered to a `RustBuffer` (a UniFFI type that stores a utf8 bytes)

The generated Rust code will then lift `Url` using the reverse of those steps.

By default, this type will appear as a string to foreign consumers.
However, each foreign language can be configured to execute a process, for example by converting the string to a `java.net.URL` in Kotlin.
This would mean that `Url` would be:

* Represented by the `url::Url` type in Rust
* Passed across the FFI as a string
* Represented by the `java.net.URL` type in Kotlin

## Custom types in the scaffolding code

### custom_type!

Use the `custom_type!` macro to define a new custom type.

```rust

// Some complex struct that can be serialized/deserialized to a string.
// This example assumes that Into/TryInto are implemented using the
// serialisation code.
use some_mod::SerializableStruct;

// When passing `SerializableStruct` objects to the foreign side, they will
// be converted to a `String`, then `String` will be used as the bridge type
// to pass the value across the FFI. Conversely, when objects are passed to
// Rust, they will be passed across the FFI as a String then converted to
// `SerializableStruct`.
uniffi::custom_type!(SerializableStruct, String);
```

Default conversions to the bridge type:

- Values passed to the foreign code will be converted using `Into<String>` then lowered as a `String` value.
- Values passed to the Rust code will lifted as a `String` then converted using `<String as TryInto<SerializableStruct>>`.
- The `TryInto::Error` type can be anything that implements `Into<anyhow::Error>`.
- `TryFrom<String>` and `From<SerializableStruct>` will also work, using the blanket impl from the core library. 
ue
### custom_type! with manual conversions

You can also manually specify the conversions by passing extra params to the
macro.   Use this when the trait implementations do not exist, or they aren't
desirable for some reason.

```rust
uniffi::custom_type!(SerializableStruct, String, {
    lower: |s| s.serialize(),
    try_lift: |s| s.deserialize(),
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
    lower: |r| LogRecordData {
        level: r.level(),
        message: r.to_string(),
    }
    try_lift: |r| LogRecord::builder()
        .level(r.level)
        .args(format_args!("{}", r.message))
        .build()
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

## Using custom types from other crates

To use custom types from other crates, use a typedef wrapped with the `[External]` attribute.
For example, if another crate wanted to use the examples above:

```idl
[External="crate_defining_handle_name"]
typedef i64 Handle;

[External="crate_defining_log_record_name"]
typedef dictionary LogRecord;
```
