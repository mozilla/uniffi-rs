# Configuration

The generated Swift module can be configured using a `uniffi.toml` configuration file.

## Available options

The configurations prefixed with `experimental_` should be regarded as unstable and
more likely to change than other configurations.

| Configuration name                  | Default                  | Description                                                                                                                                                        |
| ----------------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `cdylib_name`                       | `uniffi_{namespace}`[^1] | The name of the compiled Rust library containing the FFI implementation (not needed when using `generate --library`).                                              |
| `module_name`                       | `{namespace}`[^1]        | The name of the Swift module containing the high-level foreign-language bindings.                                                                                  |
| `ffi_module_name`                   | `{module_name}FFI`       | The name of the lower-level C module containing the FFI declarations.                                                                                              |
| `ffi_module_filename`               | `{ffi_module_name}`      | The filename stem for the lower-level C module containing the FFI declarations.                                                                                    |
| `generate_module_map`               | `true`                   | Whether to generate a `.modulemap` file for the lower-level C module with FFI declarations. (ignored by `uniffi-bindgen-swift`)                            |
| `omit_argument_labels`              | `false`                  | Whether to omit argument labels in Swift function definitions.                                                                                                     |
| `generate_immutable_records`        | `false`                  | Whether to generate records with immutable fields (`let` instead of `var`).                                                                                        |
| `experimental_sendable_value_types` | `false`                  | Whether to mark value types as `Sendable'.                                                                                                                         |
| `custom_types`                      |                          | A map which controls how custom types are exposed to Swift. See the [custom types section of the manual](../types/custom_types.md#custom-types-in-the-bindings-code) |
| `omit_localized_error_conformance`  | `false`                  | Whether to make generated error types conform to `LocalizedError`. |
| `error_enum_use_lower_camel_case`   | `false`                  | Whether to use lower camel case for error enum variants. |

[^1]: `namespace` is the top-level namespace from your UDL file.

## Example

```toml
[bindings.swift]
cdylib_name = "mycrate_ffi"
omit_argument_labels = true
```
