# Renaming

UniFFI provides two ways to customize names in generated bindings:

1. **Proc-macro attributes** - Use variants of `#[uniffi(name = "...")]` to rename items directly in Rust code. See [proc-macro renaming](proc_macro/renaming.md).
2. **TOML configuration** - Define language-specific renames in `uniffi.toml`.

## TOML-based Renaming

You can rename types, functions, methods, constructors, and their members (fields, variants, arguments) on a per-language basis using `uniffi.toml`.
The intended use-case here is for when a name could be made more idiomatic in one specific binding - you can rename many things unconditionally for all bindings using proc macros.

An example `uniffi.toml` which is renaming Python:
```toml
[bindings.python.rename]
# Rename types
# `struct MyRecord { .. }` -> `PythonRecord`
MyRecord = "PythonRecord"

# Rename nested items using dot notation
# `struct MyRecord { field: u32  }` -> `PythonRecord(python_field ...)`
"MyRecord.field" = "python_field"
"MyEnum.VariantA" = "PythonVariantA"
"MyEnum.VariantA.int_field" = "python_field"

# `fn my_function(first_arg: u8)` -> `def python_function(python_arg)`
"my_function" = "python_function"
"my_function.first_arg" = "python_arg"
"MyObject.method" = "python_method"
"MyObject.method.foo" = "python_foo"
```

The same pattern applies to all renameable items: records, record fields, enums, enum variants, enum variant fields, objects/callback interfaces/traits, and all "callables" and arguments.

### Notes

- Each crate defines its own rename configuration, you cannot rename types from external crates
- uniffi normalizes names for each language (eg, `my_func` becomes `myFunc` in some languages) after the renaming is applied.
  For example, renaming `my_func` to `renamed_func` would cause the final name to be `renamedFunc` in those languages.
- All builtin bindings support this but external bindings may not.
- Renaming the primary constructor "works", but will have no impact in bindings as the name isn't used.
