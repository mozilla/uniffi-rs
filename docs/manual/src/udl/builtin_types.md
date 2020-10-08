# Built-in types

The following built-in types can be passed as arguments/returned by Rust methods:

| Rust type            | UDL type               | Notes                             |
|----------------------|------------------------|-----------------------------------|
| `bool`               | `boolean`              |                                   |
| `u8/i8..u64/i64`     | `u8/i8..u64/i64`       |                                   |
| `f32`                | `float`                |                                   |
| `f64`                | `double`               |                                   |
| `String`             | `string`               |                                   |
| `&T`                 | `[ByRef] T`            | This works for `&str` and `&[T]`  |
| `Option<T>`          | `T?`                   |                                   |
| `Vec<T>`             | `sequence<T>`          |                                   |
| `HashMap<String, T>` | `record<DOMString, T>` | Only string keys are supported    |
| `()`                 | `void`                 | Empty return                      |
| `Result<T, E>`       | N/A                    | See [Errors](./errors.md) section |

And of course you can use your own types, which is covered in the following sections.
