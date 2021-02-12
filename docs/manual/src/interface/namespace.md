# Namespace

Every UniFFI component has an associated *namespace* which is taken from
the name of the inline module defining the interface:

```rust
#[uniffi::declare_interface]
mod example_namespace {
  // The resulting interface definition will have a namespace
  // of "example_namespace".
}
```

The namespace servces multiple purposes:
- It defines the default package/module/etc namespace in generated foreign-language bindings.
- It is used to ensure uniqueness of names in the generated C FFI.

You should ensure that the namespace of your component will be unique
amongst all UniFFI components that might be used together in an application.
