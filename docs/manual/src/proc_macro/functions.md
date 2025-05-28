# Functions, Constructors, Methods

Functions are exported to the namespace with the `#[uniffi::export]` attribute

```rust
#[uniffi::export]
fn hello_world() -> String {
    "Hello World!".to_owned()
}
```

All our owned types can be used as arguments and return types.

Arguments and receivers can also be references to these types, for example:

```rust
// Input data types as references
#[uniffi::export]
fn process_data(a: &MyRecord, b: &MyEnum, c: &Option<MyRecord>) {
    ...
}
```

To export methods of an interface you can use the [`#[uniffi::export]` attribute on an impl block](./interfaces.md).

## Default values

Exported functions/methods can have default values using the `default` argument of the attribute macro that wraps them.
`default` inputs a comma-separated list of `[name]=[value]` items.

```rust
#[uniffi::export(default(text = " ", max_splits = None))]
pub fn split(
    text: String,
    sep: String,
    max_splits: Option<u32>,
) -> Vec<String> {
  ...
}

#[derive(uniffi::Object)]
pub struct TextSplitter { ... }

#[uniffi::export]
impl TextSplitter {
    #[uniffi::constructor(default(ignore_unicode_errors = false))]
    fn new(ignore_unicode_errors: boolean) -> Self {
        ...
    }

    #[uniffi::method(default(text = " ", max_splits = None))]
    fn split(
        text: String,
        sep: String,
        max_splits: Option<u32>,
    ) -> Vec<String> {
      ...
    }
}
```

Supported default values:

  - String, integer, float, and boolean literals
  - `[]` for empty Vecs
  - `Option<T>` allows either `None` or `Some(T)`

### Renaming functions, methods and constructors

A single exported function can specify an alternate name to be used by the bindings by specifying a `name` attribute.

```rust
#[uniffi::export(name = "something")]
fn do_something() {
}
```
will be exposed to foreign bindings as a namespace function `something()`

You can also rename constructors and methods:
```rust
#[uniffi::export]
impl Something {
    // Set this as the default constructor by naming it `new`
    #[uniffi::constructor(name = "new")]
    fn make_new() -> Arc<Self> { ... }

    // Expose this as `obj.something()`
    #[uniffi::method(name = "something")]
    fn do_something(&self) { }
}
```
