# Throwing errors

It is often the case that a function does not return `T` in Rust but `Result<T, E>` to reflect that it is fallible.  
For uniffi to expose this error, your error type (`E`) must be an `enum` and implement `std::error::Error` ([thiserror](https://crates.io/crates/thiserror) works!).

Here's how you would write a Rust failible function and how you'd expose it in UDL:

```rust
#[derive(Debug, thiserror::Error)]
enum ArithmeticError {
    #[error("Integer overflow on an operation with {a} and {b}")]
    IntegerOverflow { a: u64, b: u64 },
}

fn add(a: u64, b: u64) -> Result<u64, ArithmeticError> {
    a.checked_add(b).ok_or(ArithmeticError::IntegerOverflow { a, b })
}
```

And in UDL:

```
[Error]
enum ArithmeticError {
  "IntegerOverflow",
};


namespace arithmetic {
  [Throws=ArithmeticError]
  u64 add(u64 a, u64 b);
}
```

On the other side (Kotlin, Swift etc.), a proper exception will be thrown if `Result::is_err()` is `true`.
