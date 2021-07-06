# Functions

All top-level *functions* get exposed through the UDL's `namespace` block.
For example, if the crate's `lib.rs` file contains:

```rust
fn hello_world() -> String {
    "Hello World!".to_owned()
}
```

The UDL file will look like:

```idl
namespace Example {
    string hello_world();
}
```

## Optional arguments & default values

Function arguments can be marked `optional` with a default value specified.

In the UDL file:

```idl
namespace Example {
    string hello_name(optional string name = "world");
}
```

The Rust code will take a required argument:

```rust
fn hello_name(name: String) -> String {
    format!("Hello {}", name)
}
```

The generated foreign-language bindings will use function parameters with default values.
This works for the Kotlin, Swift and Python targets.

For example the generated Kotlin code will be equivalent to:

```kotlin
fun helloName(name: String = "world" ): String {
    // ...
}
```
