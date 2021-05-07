# Record Structs

Records can be compared to POJOs in the Java world: just a data structure holding some data.
In the interface definition, they are distinguished from Object structs by the presence of
public fields:

```rust
#[uniffi_macros::declare_interface]
mod todolist {
    struct TodoEntry {
        pub done: bool,
        pub due_date: u64,
        pub text: String,
    }
}
```

Records can contain other records and every other data type available, except objects.
(Although they cannot currently contain themselves recursively).
