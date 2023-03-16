# Built-in types

The following built-in types can be passed as arguments/returned by Rust methods:

| Rust type            | UDL type               | Notes                                                           |
|----------------------|------------------------|-----------------------------------------------------------------|
| `bool`               | `boolean`              |                                                                 |
| `u8/i8..u64/i64`     | `u8/i8..u64/i64`       |                                                                 |
| `f32`                | `float`                |                                                                 |
| `f64`                | `double`               |                                                                 |
| `String`             | `string`               |                                                                 |
| `SystemTime`         | `timestamp`            | Precision may be lost when converting to Python and Swift types |
| `Duration  `         | `duration`             | Precision may be lost when converting to Python and Swift types |
| `&T`                 | `[ByRef] T`            | This works for `&str` and `&[T]`                                |
| `Option<T>`          | `T?`                   |                                                                 |
| `Vec<T>`             | `sequence<T>`          |                                                                 |
| `HashMap<String, T>` | `record<DOMString, T>` | Only string keys are supported                                  |
| `()`                 | `void`                 | Empty return                                                    |
| `Result<T, E>`       | N/A                    | See [Errors](./errors.md) section                               |
| `uniffi::TaskQueue`  | `TaskQueue`            | Task queue in the foreign language: Kotlin CoroutineScope, Python Executor, Swift DispatchQueue, etc. |

And of course you can use your own types, which is covered in the following sections.
