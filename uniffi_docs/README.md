This crate contains functionality related to documentation comments generation 
based on the `lib.rs` Rust binding source code accompanying `.udl` file
specifying the interface.

Documentation comments generation is disabled by default and can be enabled in `uniffi.toml` file:

```toml
[bindings]
doc_comments = true
```

See `documentation` example for reference.
