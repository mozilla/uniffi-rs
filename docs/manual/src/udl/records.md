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

## Optional and compound fields.

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

You can similarly use a `HashMap`, `Vec<>` etc.

```idl
dictionary TodoEntry {
    sequence<string> item_list;
    record<string, string> item_map;
};
```

## Default values for fields

Fields can be specified with a [default literal value](../types/defaults.md#literal-values).

```idl
dictionary TodoEntry {
    boolean done = false;
    string text = "unnamed";
};
```

An `Option<>` can be specified as `null` or an appropriate literal for a `Some` value. `HashMap` and `Vec` have custom syntax.

```idl
dictionary TodoEntry {
    string? text = null;
    string? alt_text = "unnamed"
    sequence<string> item_list = [];
    record<string, string> item_map = {};
};
```

