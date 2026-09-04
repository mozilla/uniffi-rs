# Configuration

The generated Ruby modules can be configured using a `uniffi.toml` configuration file.

## Available options

| Configuration name | Default  | Description |
| ------------------ | -------- | ----------- |
| `cdylib_name`      | `uniffi_{namespace}`[^1] | The name of the compiled Rust library containing the FFI implementation (not needed when using `generate --library`). |
| `cdylib_path`      | | An explicit path to the shared library, passed as the `ffi_lib` argument. |
| `module_name`      | `UpperCamelCase(namespace)` | The Ruby module name emitted as `module …` in that crate's generated bindings. Must be a single constant identifier (e.g. `UniffiOne`), not a nested path (`Foo::Bar`). |
| `custom_types`     | | A map which controls how custom types are exposed to Ruby. See the [custom types section of the manual](../types/custom_types.md#custom-types-in-the-bindings-code) |
| `external_packages` | | A map from Rust crate names to Ruby module names for use with [external types](../types/remote_ext_types.md). Keys are crate names, not UniFFI namespaces: the Cargo package name (`my-crate`) and the underscored Rust crate name (`my_crate`) are equivalent (`uniffi_one`, not `uniffi_one_ns`). This controls how generated code *references* external modules (e.g. `OtherCrate::SomeType`). `require` paths still use each crate's UniFFI namespace (the generated `.rb` filename). In library mode (`generate --library`) every peer crate is auto-filled from that crate's `module_name`; a listed value must match. A mismatch is a bindgen error. Omitting a key does not skip the crate or control mixin / `require` membership. |
| `rename`           | | A map to rename types, functions, methods, and their members in the generated Ruby bindings. See the [renaming section](../renaming.md). |
| `exclude`          | | A list of crate names to exclude when generating bindings for a library (library mode). |

[^1]: The namespace is derived from the crate name or UDL file name.

## Prerequisites

Ruby bindings require the [`ffi` gem](https://github.com/ffi/ffi). See [docs/contributing.md](https://github.com/mozilla/uniffi-rs/blob/main/docs/contributing.md) for setup instructions.

## Examples

Custom Types:

```toml
# Assuming a Custom Type named Url using a String as the builtin.
[bindings.ruby.custom_types.Url]
type_name = "URI"
imports = ["uri"]
lift = "URI.parse({})"
lower = "{}.to_s"
```

Module name (defining crate):

```toml
[bindings.ruby]
# Default is UpperCamelCase of the UniFFI namespace (e.g. uniffi_one_ns → UniffiOneNs).
module_name = "UniffiOne"
```

External Packages (consumer; optional in library mode):

```toml
[bindings.ruby.external_packages]
# Map the crate name from [External={name}] / Cargo.toml to its Ruby module name.
# Hyphens and underscores are equivalent: `rust-crate-name` and `rust_crate_name` are the same key.
# Library mode already maps every peer crate to that crate's `module_name`; a listed
# value must match. Keys are crate names, not namespaces.
rust-crate-name = "ExternalRubyModule"
```

Refer to [`examples/custom-types/uniffi.toml`](https://github.com/mozilla/uniffi-rs/blob/main/examples/custom-types/uniffi.toml) for a complete example.

## InternalError

Each generated module defines its own `InternalError` class (`StandardError` subclass), for example `Coverall::InternalError`.
This is the same idea as Python `InternalError` and Kotlin `InternalException`: a public bindings-level error for panics, protocol mismatches, and corrupt buffers — not for declared API errors.

Readers and writers for a type live in that type's crate (Ruby mixins, like Python/Kotlin converters).
A corrupt buffer while lifting an [external type](../types/remote_ext_types.md) therefore raises **the defining crate's** `InternalError`.
`rescue ImportedTypesLib::InternalError` will not catch `UniffiOneNs::InternalError`.

Rescue the crate that owns the type, rescue each crate you depend on, or use `rescue StandardError` as a catch-all.
