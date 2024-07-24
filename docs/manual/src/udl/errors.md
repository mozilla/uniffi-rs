# Throwing errors

It is often the case that a function does not return `T` in Rust but `Result<T, E>` to reflect that it is fallible.  
For UniFFI to expose this error, your error type (`E`) must be an `enum` and implement `std::error::Error` ([thiserror](https://crates.io/crates/thiserror) works!).

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

If you want to expose the associated data as fields on the exception, use this syntax:

```
[Error]
interface ArithmeticError {
  IntegerOverflow(u64 a, u64 b);
};
```

## Interfaces as errors

It's possible to use an `interface` (ie, a rust struct impl or a `dyn Trait`) as an error;
the thrown object will have methods instead of fields.
This can be particularly useful when working with `anyhow` style errors, where
an enum can't easily represent certain errors.

In your UDL:
```
namespace error {
  [Throws=MyError]
  void bail(string message);
}

[Traits=(Debug)]
interface MyError {
  string message();
};
```
and Rust:
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
