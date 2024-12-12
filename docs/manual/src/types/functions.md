# Functions

Functions are exposed in a [namespace](./namespace.md), either via [UDL](../udl/functions.md) or [proc-macros](../proc_macro/functions.md)

```rust
#[uniffi::export] // if using proc-macros
fn hello_world() -> String {
    "Hello World!".to_owned()
}
```

Note that everything described here applies to all "callables" - eg, interface methods, constructors etc.

## Optional arguments & default values

You can specify a default value for function arguments (in [UDL](../udl/functions.md#default-values) and [proc-macros](../proc_macro/functions.md#default-values))

The Rust code will still take a required argument:

```rust
fn hello_name(name: String) -> String {
    format!("Hello {}", name)
}
```

The generated foreign-language bindings will use function parameters with default values.

If the default argument value is `"world"`, the generated Kotlin code will be equivalent to:

```kotlin
fun helloName(name: String = "world" ): String {
    // ...
}
```

## Async

Async functions "just work" with proc-macros, while [UDL can use the `[Async]`](../udl/functions.md#async) attribute:

See the [Async/Future support section](../futures.md) for details.
