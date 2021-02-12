# Built-in types

The following built-in types can be passed as arguments/returned by Rust methods:

| Rust type               | Notes                             |
|-------------------------|-----------------------------------|
| `bool`                  |                                   |
| `u8/i8..u64/i64`        |                                   |
| `f32`                   |                                   |
| `f64`                   |                                   |
| `String`                |                                   |
| `&T`                    | This works for `&str` and `&[T]`  |
| `Option<T>`             |                                   |
| `Vec<T>`                |                                   |
| `HashMap<String, T>`    | Only string keys are supported    |
| `()`                    | Empty return                      |
| `Result<T, E>`          | See [Errors](./errors.md) section |
| `std::time::SystemTime` |                                   |
| `std::time::Duration`   |                                   |

And of course you can use your own types, which is covered in the following sections.

UniFFI does not currently support type aliases, so you cannot do e.g. `type handle = u64`
and then use `handle` as the argument type of your exposed functions. This limitation
may be removed in future.
