# Docstrings

In proc-macros, Rust docstrings will be captured and rendered in the bindings.

For example:
```rust
/// This is the docstring for MyObject
#[derive(uniffi::Object)]
pub struct MyObject {}
```

Will cause Python, Swift and Kotlin to all generate a wrapper for `MyObject` with appropriate docstrings for that language.

You can see examples of how they are rendered in the [UDL docstrings documentation](../udl/docstrings.md)

## Custom Types

Note that [Custom Types](../types/custom_types.md) need a different syntax - the docstring must be in the macro invocation. eg:

```rust
uniffi::custom_newtype!(
    /// This is a docstring for Handle
    Handle, i64
);
```

For more info, see [the custom types documentation](../types/custom_types.md#custom_type-docstrings)
