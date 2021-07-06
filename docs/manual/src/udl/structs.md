# Structs/Dictionaries

Dictionaries are how UniFFI represents structured data.
They consist of one of more named *fields*, each of which holds a value of a particular type.
Think of them like a Rust struct without any methods.

A Rust struct like this:

```rust
struct TodoEntry {
    done: bool,
    due_date: u64,
    text: String,
}
```

Can be exposed via UniFFI using UDL like this:

```idl
dictionary TodoEntry {
    boolean done;
    u64 due_date;
    string text;
};
```

The fields in a dictionary can be of almost any type, including objects or other dictionaries.
The current limitations are:

* They cannot recursively contain another instance of the *same* dictionary type.
* They cannot contain references to callback interfaces.

## Fields holding Object References

If a dictionary contains a field whose type is an [interface](./interfaces.md), then that
field will hold a *reference* to an underlying instance of a Rust struct. The Rust code for
working with such fields must store them as an `Arc` in order to help properly manage the
lifetime of the instance. So if the UDL interface looked like this:

```idl
interface User {
    // Some sort of "user" object that can own todo items
};

dictionary TodoEntry {
    User owner;
    string text;
}
```

Then the corresponding Rust code would need to look like this:

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

Fields can be specified with a default value:

```idl
dictionary TodoEntry {
    boolean done = false;
    string text;
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
If not set otherwise the default value for a field is passed to the Rust constructor.

## Optional fields and default values

Fields can be made optional using a `T?` type.

```idl
dictionary TodoEntry {
    boolean done;
    string? text;
};
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

Optional fields can also be set to a default `null` value:

```idl
dictionary TodoEntry {
    boolean done;
    string? text = null;
};
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
