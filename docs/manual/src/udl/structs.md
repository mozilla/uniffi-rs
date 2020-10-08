# Structs/Dictionaries

Dictionaries can be compared to POJOs in the Java world: just a data structure holding some data.

```rust
struct TodoEntry {
    done: bool,
    due_date: u64,
    text: String,
}
```

can be converted in UDL to:

```idl
dictionary TodoEntry {
    boolean done;
    u64 due_date;
    string text;
};

```

Dictionaries can contain each other and every other data type available, except objects.
