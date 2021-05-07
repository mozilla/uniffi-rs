# Functions

The interface definition can contain public functions using the standard Rust syntax:

```rust
#[uniffi_macros::declare_interface]
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

#[uniffi_macros::declare_interface]
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
