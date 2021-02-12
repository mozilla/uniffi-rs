# Record Structs

Dictionaries are how UniFFI represents structured data.
They consist of one of more named *fields*, each of which holds a value of a particular type.

In the interface definition, they are distinguished from Object structs by the presence of
public fields:

```rust
#[uniffi::declare_interface]
mod todolist {
    struct TodoEntry {
        pub done: bool,
        pub due_date: u64,
        pub text: String,
    }
}
```

The fields in a record can be of almost any type, including objects or other dictionaries.
The current limitations are:

* They cannot recursively contain another instance of the *same* record type.
* They cannot contain references to callback interfaces.

These restrictions may be lifted in future.

## Fields holding Object References

If a record contains a field whose type is an [object](./objects.md), then that
field will hold a *reference* to an underlying instance of a Rust struct. The Rust code for
working with such fields must store them as an `Arc` in order to help properly manage the
lifetime of the instance. Like this:

```rust,no_run
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

Fields can be specified with a default value:

TODO: what syntax will we use for this in Rust? Some sort of helper macro attribute like:

```rust,no_run
pub struct TodoEntry {
    #[uniffi(default=false)]
    done: bool,
    text: String,
};
```

The corresponding generated Kotlin code would be equivalent to:

```kotlin
data class TodoEntry (
    var done: Boolean = false,
    var text: String
)  {
    // ...
}
```

This works for Swift and Python targets too.
If not set otherwise the default value for a field is used when constructing the Rust struct.

## Optional fields and default values

Fields can be made optional using Rust's builtin `Option` type:

```rust,no_run
pub struct TodoEntry {
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

Optional fields can also be set to a default `None` value:

```rust,no_run
pub struct TodoEntry {
    done: bool,
    #[uniffi(default=None)]
    text: Option<String>,
}
```

The corresponding generated Kotlin code would be equivalent to:

```kotlin
data class TodoEntry (
    var done: Boolean,
    var text: String? = null
)  {
    // ...
}
```

This works for Swift and Python targets too.