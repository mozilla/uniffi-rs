# Throwing errors

It is often the case that a function does not return `T` in Rust but `Result<T, E>` to reflect that it is fallible.  
For UniFFI to expose this error, your error type (`E`) must be an `enum` and implement `std::error::Error` ([thiserror](https://crates.io/crates/thiserror) works!).

Errors must be exposed via [UDL](../udl/errors.md) or [proc-macros](../proc_macro/errors.md)

Here's how you would write a Rust failible function:

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

On the other side (Kotlin, Swift etc.), a proper exception will be thrown if `Result::is_err()` is `true`.

## Interfaces as errors

It's possible to use an `interface` (ie, a rust struct impl or a `dyn Trait`) as an error;
the thrown object will have methods instead of fields.
This can be particularly useful when working with `anyhow` style errors, where
an enum can't easily represent certain errors.

```rs
#[derive(Debug, thiserror::Error)]
#[error("{e:?}")] // default message is from anyhow.
pub struct MyError {
    e: anyhow::Error,
}

impl MyError {
    fn message(&self) -> String { self.to_string() }
}

impl From<anyhow::Error> for MyError {
    fn from(e: anyhow::Error) -> Self {
        Self { e }
    }
}
```

You can't yet use `anyhow` directly in your exposed functions - you need a wrapper:

```rs
fn oops() -> Result<(), MyError> {
    let e = anyhow::Error::msg("oops");
    Err(e.into())
}
```
then in Python:
```py
try:
  oops()
except MyError as e:
  print("oops", e.message())
```

This works for procmacros too - just derive or export the types.
```rs
#[derive(Debug, uniffi::Error)]
pub struct MyError { ... }
#[uniffi::export]
impl MyError { ... }
#[uniffi::export]
fn oops(e: String) -> Result<(), Arc<MyError>> { ... }
```

[See our tests this feature.](https://github.com/mozilla/uniffi-rs/tree/main/fixtures/error-types)
