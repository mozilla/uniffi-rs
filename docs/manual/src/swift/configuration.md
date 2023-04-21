# Configuration

The generated Swift module can be configured using a `uniffi.toml` configuration file.

## Available options

| Configuration name | Default  | Description |
| ------------------ | -------  |------------ |
| `cdylib_name`      | `uniffi_{namespace}`[^1] | The name of the compiled Rust library containing the FFI implementation (not needed when using `generate --crate`). |
| `module_name`      | `{namespace}`[^1] | The name of the Swift module containing the high-level foreign-language bindings. |
| `ffi_module_name`  | `{module_name}FFI` | The name of the lower-level C module containing the FFI declarations. |
| `ffi_module_filename` | `{ffi_module_name}` | The filename stem for the lower-level C module containing the FFI declarations. |
| `generate_module_map` | `true` | Whether to generate a `.modulemap` file for the lower-level C module with FFI declarations. |
| `omit_argument_labels` | `false` | Whether to omit argument labels in Swift function definitions. |

[^1]: `namespace` is the top-level namespace from your UDL file.

## Example

```toml
[bindings.swift]
cdylib_name = "mycrate_ffi"
omit_argument_labels = true
```
