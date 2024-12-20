# Records in UDL

[Our simple TodoEntry](../types/records.md) is defined in UDL as:

```idl
dictionary TodoEntry {
    boolean done;
    u64? due_date;
    string text;
};
```

All the usual types are supported.

# Object references

Our dictionary can refer to obects - here, a `User`

```idl
interface User {
    // Some sort of "user" object that can own todo items
};

dictionary TodoEntry {
    User owner;
    string text;
}
```

The Rust struct will have `owner` as an `Arc<>`.

## Default values for fields

Fields can be specified with a default value.

```idl
dictionary TodoEntry {
    boolean done = false;
    string text;
};
```

## Optional fields and default values

Fields can be made optional using a `T?` type.

```idl
dictionary TodoEntry {
    boolean done;
    string? text;
};
```

The corresponding Rust struct would need to look like this:

```rust
struct TodoEntry {
    done: bool,
    text: Option<String>,
}
```

### Optional null values

Optional fields can also be set to a default `null` value:

```idl
dictionary TodoEntry {
    boolean done;
    string? text = null;
};
```

