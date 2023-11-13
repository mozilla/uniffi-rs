# Configuration

The generated Python modules can be configured using a `uniffi.toml` configuration file.

## Available options

| Configuration name | Default  | Description |
| ------------------ | -------  |------------ |
| `cdylib_name`      | `uniffi_{namespace}`[^1] | The name of the compiled Rust library containing the FFI implementation (not needed when using `generate --library`). |
| `custom_types`      | | A map which controls how custom types are exposed to Python. See the [custom types section of the manual](../udl/custom_types.md#custom-types-in-the-bindings-code)|
| `external_packages` | | A map which controls the package name used by external packages. See below for more.

## External Packages

When you reference external modules, uniffi will generate statements like `from module import Type`
in the referencing module. The `external_packages` configuration value allows you to specify how `module`
is formed in such statements.

The value is a map, keyed by the crate-name and the value is the package name which will be used by
Python for that crate. The default value is an empty map.

When looking up crate-name, the following behavior is implemented.

### Default value
If no value for the crate is found, it is assumed that you will be packaging up your library
as a simple Python package, so the statement will be of the form `from .module import Type`,
where `module` is the namespace specified in that crate.

Note that this is invalid syntax unless the module lives in a package - attempting to
use the module as a stand-alone module will fail. UniFFI just generates flat .py files; the
packaging is up to you. Eg, a build process might create a directory, create an `__init__.py`
file in that directory (maybe including `from subpackage import *`) and have `uniffi-bindgen`
generate the bindings into this directory.

### Specified value
If the crate-name is found in the map, the specified entry used as a package name, so the statement will be of the form
`from package.module import Type` (again, where `module` is the namespace specified in that crate)

An exception is when the specified value is an empty string, in which case you will see
`from module import Type`, so each generated module functions outside a package.
This is used by some UniFFI tests to avoid the test code needing to create a Python package.

## Examples

Custom Types
```toml
# Assuming a Custom Type named URL using a String as the builtin.
[bindings.python.custom_types.Url]
imports = ["urllib.parse"]
# Functions to convert between strings and the ParsedUrl class
into_custom = "urllib.parse.urlparse({})"
from_custom = "urllib.parse.urlunparse({})"
```

External Packages
```toml
[bindings.python.external_packages]
# An external type `Foo` in `crate-name` (which specifies a namespace of `my_module`) will be referenced via `from MyPackageName.my_module import Foo`
crate-name = "MyPackageName"
```
