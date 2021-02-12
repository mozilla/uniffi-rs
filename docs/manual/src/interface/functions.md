# Functions

The interface definition can contain public functions using the standard Rust syntax:

```rust
#[uniffi::declare_interface]
mod example {
    fn hello_world() -> String {
        "Hello World!".to_owned()
    }
}
```

The arguments and return types must be types that are understood by UniFFI.

UniFFI does not understand path references or type aliases, so things like
the following will produce a compile-time error:

```rust

struct NotPartOfTheInterface {
    value: u32,
}

#[uniffi::declare_interface]
mod example {

    type MyString = String

    // Error: UniFFI doesn't know what "MyString" is.
    fn hello_world() -> MyString {
        "Hello World!".to_owned()
    }

    // Error: UniFFI doesn't know about the `NotPartOfTheInterface` type.
    fn example(v: NotPartOfTheInterface) {
        println!("value: {}", v.value);
    }

    // Error: UniFFI doesn't understand the `std::vec::` path; just use `Vec<T>`.
    fn smallest(values: std::vec::Vec<i32>) -> i32 {
        values.iter().min()
    }
}
```

## Optional arguments & default values

Function arguments can be marked as optional with a default value specified.
TODO: what will the Rust syntax for this be?

The Rust code will declare this using a macro annotation:

```rust
#[uniffi::defaults(name="World")] // TODO: not even sure what syntax is possible here...
fn hello_name(name: String) -> String {
    format!("Hello {}", name)
}
```

The generated foreign-language bindings will use function parameters with default values.
This works for the Kotlin, Swift and Python targets.

For example the generated Kotlin code will be equivalent to:

```kotlin
fun helloName(name: String = "World" ): String {
    // ...
}
```