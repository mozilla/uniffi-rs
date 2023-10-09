# Configuration

The generated Python modules can be configured using a `uniffi.toml` configuration file.

## Available options

| Configuration name | Default  | Description |
| ------------------ | -------  |------------ |
| `cdylib_name`      | `uniffi_{namespace}`[^1] | The name of the compiled Rust library containing the FFI implementation (not needed when using `generate --library`). |
| `custom_types`      | | A map which controls how custom types are exposed to Python. See the [custom types section of the manual](../udl/custom_types#custom-types-in-the-bindings-code)|


## Example

```toml
# Assuming a Custom Type named URL using a String as the builtin.
[bindings.python.custom_types.Url]
imports = ["urllib.parse"]
# Functions to convert between strings and the ParsedUrl class
into_custom = "urllib.parse.urlparse({})"
from_custom = "urllib.parse.urlunparse({})"
```
