# Namespace

Every UDL file *must* have a `namespace` block:

```idl
namespace math {
  double exp(double a);
};
```

It serves multiple purposes:
- It identifies the name of the generated Rust scaffolding file `<namespace>.uniffi.rs`.
- It identifies the package name of the generated foreign-language bindings (e.g. `uniffi.<namespace>` in Kotlin)
- It also contains all [top-level *functions*](./functions.md) that get exposed to foreign-language bindings.
