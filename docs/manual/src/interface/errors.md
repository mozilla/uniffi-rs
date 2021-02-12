# Throwing errors

It is often the case that a function does not return `T` in Rust but `Result<T, E>` to reflect that it is fallible. UniFFI can expose such functions as long as your error type `E` meets the following requirements:

* It is a `pub enum` exposed as part of the interface definition (see [enumerations](./enumerations.md))
* It impls the `std::error::Error` trait

(Using [thiserror](https://crates.io/crates/thiserror) works!).

Here's how you would write a Rust failible function:

```rust

#[uniffi::declare_interface]
mod example {

  use thiserror::Error;

  #[derive(Debug, Error)]
  pub enum ArithmeticError {
      #[error("Integer overflow on an operation with {a} and {b}")]
      IntegerOverflow { a: u64, b: u64 },
  }

  pub fn add(a: u64, b: u64) -> Result<u64, ArithmeticError> {
      a.checked_add(b).ok_or(ArithmeticError::IntegerOverflow { a, b })
  }
}
```

Note that you cannot currently use a typedef for the `Result` type, UniFFI only supports
the full `Result<T, E>` syntax. This limitation may be removed in future.

The resulting `add` function would be exposed to the foreign-language code using
its native error-handling mechanism. For example:

* In Kotlin, there would be an `ArithmeticException` class with an inner class
  for each variant, and the `add` function would throw it.
* In Swift, there would be an `ArithmeticError` enum with variants matching the Rust enum,
  and the `add` function would be marked `throws` and could throw it.
