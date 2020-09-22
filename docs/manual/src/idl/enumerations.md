# Enumerations

An enumeration defined in Rust code as
```rust
enum Animal {
    Dog,
    Cat,
}
```

Can be exposed in the IDL file with:

```idl
enum Animal {
  "Dog",
  "Cat",
};
```

Note that enumerations with associated data are not yet supported.
