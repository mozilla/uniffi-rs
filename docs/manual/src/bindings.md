# Generating bindings

"Bindings" is the term for the non-Rust side of the environment - the generated Python, Swift or Kotlin code which drives the examples.

UniFFI comes with a `uniffi_bindgen` which generates these bindings. For introductory
information, see [Foreign Language Bindings in the tutorial](./tutorial/foreign_language_bindings.md).
There are also external bindings which don't live in this repo.

# Customizing the binding generation

Each binding generator has their own unique configuration options to customize their output.
There's a toml-based capability to specify these various options.

In the simple default configuration, the bindings read a file `uniffi.toml` in the root of each crate.
There's also a global configuration option, useful when multiple crates need to share some settings.
This configuration has the same options as available in `uniffi.toml`.

Of the builtin bindings, see [Kotlin](./kotlin/configuration.md), [Python](./python/configuration.md) and [Swift](./swift/configuration.md).

# Global configuration

For projects with multiple UniFFI crates, a global config file lets you set shared defaults and
per-crate overrides in one place. For example:

```
uniffi-bindgen generate --config global.toml -l swift -o out/ libmylibrary.dylib
```

The file has three optional sections:

- **`[crate-roots]`** — Maps crate names to their root directories. Use this when
  `cargo metadata` is unavailable or to override crate locations. Paths are relative
  to the config file's directory.

- **`[defaults]`** — Configuration applied to every crate, using the same format as
  `uniffi.toml`. These are the lowest priority and are overridden by a crate's own
  `uniffi.toml`.

- **`[crates.<name>]`** — Per-crate overrides with the highest priority. These override
  both `[defaults]` and the crate's own `uniffi.toml`.

**Priority** (lowest to highest): `[defaults]` → crate's `uniffi.toml` → `[crates.<name>]`

All sections use the same format as `uniffi.toml`. Tables are merged recursively; scalar
values replace.

### Example: shared FFI module name with per-crate filenames

```toml
# ffi_module_name must be the same for all crates
[defaults.bindings.swift]
ffi_module_name = "MyAppFFI"

# ffi_module_filename must differ per crate
[crates.auth.bindings.swift]
ffi_module_filename = "auth_ffi"

[crates.storage.bindings.swift]
ffi_module_filename = "storage_ffi"
```

# Crate locations

UniFFI needs to locate UDL files and `uniffi.toml` files for each referenced crate.
It uses the same mechanism in both places - either via the `cargo-metadata` feature,
and/or via global configuration.
See the discussion about `[crate-roots]` above for more info.
If there's no `[crate-roots]` entry for a crate, we'll try and fall back to using `cargo-metadata`
if it's enabled, otherwise we'll use the default config for the crate (and give an error if we need a UDL).

### Example: locating crates without cargo-metadata

```toml
[crate-roots]
# These entries also tell UniFFI where to find `suggest.udl` or `uniffi.toml` files,
# avoiding cargo-metadata.
suggest = "./components/suggest"
tabs = "./components/tabs"

# This same file can also have `[defaults]` or overrides, like:
[defaults.bindings.swift]
some_global_option = true # default for every crate

[crates.suggest.bindings.swift]
some_option = true # specific override for the `suggest` crate.
```
