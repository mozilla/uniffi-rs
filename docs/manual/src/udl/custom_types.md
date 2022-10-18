# Custom types

Custom types allow you to extend the UniFFI type system to support types from your Rust crate or 3rd
party libraries.  This relies on a [builtin](./builtin_types.md) UDL type move data across the
FFI, followed by a conversion to your custom type.

## Custom types in the scaffolding code

Consider the following trivial Rust abstraction for a "handle" which wraps an integer:

```rust
pub struct Handle(i64);
```

You can use this type in your udl by declaring it via a `typedef` with a `Custom` attribute,
defining the builtin type that it's based on.

```idl
[Custom]
typedef i64 Handle;
```

For this to work, your Rust code must include the trait implementations
`impl TryFrom<Builtin> for Handle` and `impl From<Handle> for Builtin` where `Builtin` is the
Rust type corresponding to the UniFFI builtin type - `i64` in the example above.

The `Error` type of the `TryFrom` implementations has to implement the `std::error::Error` trait.
Due to Rust's blanket implementation of `TryFrom<U> for T` where `U: Into<T>`, you can also
implement the infallible `From<Builtin> for Handle` instead of the fallible `TryFrom` version.
In this case:

```rust
impl From<i64> for Handle {
    fn from(value: i64) -> Handle {
        Self(value)
    }
}

impl From<Handle> for i64 {
    fn from(value: Handle) -> i64 {
        value.0
    }
}
```

## Error handling during conversion

You might have noticed that the `into_custom` function returns a `uniffi::Result` (which is an
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
enum ExampleErrors {
    "InvalidHandle"
};

namespace errors_example {
    take_handle_1(Handle handle);

    [Throws=ExampleErrors]
    take_handle_2(Handle handle);
}
```

and the following Rust:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ExampleErrors {
    #[error("The handle is invalid")]
    InvalidHandle,
}

impl TryFrom<i64> for ExampleHandle {
    type Error = ExampleErrors;

    fn try_from(val: i64) -> Result<Self, ExampleErrors> {
        if (val == 0) {
            Err(ExampleErrors::InvalidHandle.into())
        } else {
            Ok(Handle(val))
        }
    }
    ...
}
```

The behavior of the generated scaffolding will be:

* Calling `take_handle_1` with a value of `0` will panic.
* Calling `take_handle_2` with a value of `0` will return `Err(ExampleErrors)` exception
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

## Using Custom Types from other crates

To use the `Handle` example above from another crate, these other crates just refer to the type
as a regular `External` type - for example, another crate might use `udl` such as:

```idl
[External="crate_defining_handle_name"]
typedef extern Handle;
```
