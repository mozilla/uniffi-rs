# Records

Records are how UniFFI represents structured data.
They consist of one of more named *fields*, each of which holds a value of a particular type. A rust struct without any methods.

Records were sometimes referred to as a dictionary due to the UDL syntax, but we try to use Record.

There's further specific detail for [UDL](../udl/records.md) and [proc-macros](../proc_macro/records.md)

They can be exported as a simple rust struct:

```rust
#[derive(uniffi::Record)] // if using proc-macros
struct TodoEntry {
    done: bool,
    due_date: Option<u64>,
    text: String,
}
```

The fields in a record can be of almost any type, including objects or other records.
The current limitations are:

* They cannot recursively contain another instance of the *same* record type.
* They cannot contain references to callback interfaces (which is exactly why callback interfaces should be avoided, use traits)

# Fields holding object references

A record containing an [interface](./interfaces.md) must use an `Arc<>`.

```rust
struct TodoEntry {
    owner: std::sync::Arc<User>,
    text: String,
}
```

Depending on the language, the foreign-language bindings may also need to be aware of
these embedded references. For example in Kotlin, each Object instance must be explicitly
destroyed to avoid leaking the underlying memory, and this also applies to Objects stored
in record fields.

You can read more about managing object references in the section on [interfaces](./interfaces.md).

## Default values for fields

Fields can be specified with a default value.

```rust
struct TodoEntry {
    #[uniffi(default = false)] // or specified in UDL.
    done: bool,
    text: String,
}
```

The above example shows proc-macros; see also the [UDL docs](../udl/records.md#default-values-for-fields)

The corresponding generated Kotlin code would be equivalent to:

```kotlin
data class TodoEntry (
    var done: Boolean = false,
    var text: String
)  {
    // ...
}
```

## Optional fields and default values

Fields can be made optional by using `Option<T>`

```rust
struct TodoEntry {
    done: bool,
    text: Option<String>,
}
```

The corresponding generated Kotlin code would be equivalent to:

```kotlin
data class TodoEntry (
    var done: Boolean,
    var text: String?
)  {
    // ...
}
```

The default for this can be `null/None/etc` generating:

```kotlin
data class TodoEntry (
    var done: Boolean,
    var text: String? = null
)  {
    // ...
}
```
