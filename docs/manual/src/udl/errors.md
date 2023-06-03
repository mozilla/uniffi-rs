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

** This is not avaiable in all bindings **

It's possible to use an `interface` (ie, a rust struct impl) as an error;
the thrown object will have methods instead of fields.
This can be particularly useful when working with `anyhow` style errors, where
an enumeration can't represent the various error states.

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
#[derive(Debug)]
pub struct MyError {
    e: anyhow::Error,
}

impl MyError {
    fn message(&self) -> String> { ... }
}

// anything using `anyhow::Error` can throw `MyError`.
impl From<anyhow::Error> for MyError {
    fn from(e: anyhow::Error) -> Self {
        Self { e }
    }
}

fn bail(message: String) -> anyhow::Result<()> {
    anyhow::bail!("{message}");
}
```
then Python might then do:
```py
try:
  bail("oh no")
except MyError as e:
  print("oops", e.message())
```

This works for procmacros too, although without the builtin type coercion:

```rs
#[derive(Debug, uniffi::Error)]
pub struct ProcErrorInterface {
    e: String,
}

#[uniffi::export]
impl ProcErrorInterface {
    fn message(&self) -> String {
        self.e.clone()
    }
}

impl std::fmt::Display for ProcErrorInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProcErrorInterface {}", self.e)
    }
}

#[uniffi::export]
fn throw_proc_error(e: String) -> Result<(), Arc<ProcErrorInterface>> {
    Err(Arc::new(ProcErrorInterface { e }))
}
```

Note how `throw_proc_error` must return the concrete type and can't just use
`anyhow::Error()` as the `Err`, so `throw_proc_error` would need to make
any conversions explicit.
