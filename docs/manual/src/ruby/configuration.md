# Configuration

The generated Ruby modules can be configured using a `uniffi.toml` configuration file.

## Available options

| Configuration name | Default  | Description |
| ------------------ | -------- | ----------- |
| `cdylib_name`      | `uniffi_{namespace}`[^1] | The name of the compiled Rust library containing the FFI implementation (not needed when using `generate --library`). |
| `cdylib_path`      | | An explicit path to the shared library, passed as the `ffi_lib` argument. |
| `custom_types`     | | A map which controls how custom types are exposed to Ruby. See the [custom types section of the manual](../types/custom_types.md#custom-types-in-the-bindings-code) |
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

Refer to [`examples/custom-types/uniffi.toml`](https://github.com/mozilla/uniffi-rs/blob/main/examples/custom-types/uniffi.toml) for a complete example.