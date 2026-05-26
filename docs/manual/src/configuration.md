# Customizing the bindings generation

"Bindings" are the non-Rust side of the environment.

Each binding generator has their own unique configuration options to customize their output.
There's a toml-based capability to specify these various options, allowing a project to specify all configuration options across all bindings.

We support:

* a `uniffi.toml` in the root of each crate,
* a "global configuration" for your project, as described below.

In all cases, you should see the documentation for the builtin bindings
([Kotlin](./kotlin/configuration.md), [Python](./python/configuration.md), [Swift](./swift/configuration.md)), or external bindings docs.

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

All sections use the same format as `uniffi.toml`. Tables/maps are merged recursively; scalar
values are replaced.

### Example: shared FFI module name with per-crate filenames

Imagine a `global.toml` with:

```toml
# ffi_module_name must be the same for all crates
[defaults.bindings.swift]
ffi_module_name = "MyAppFFI"

# ffi_module_filename must differ per crate, here the `auth` crate
[crates.auth.bindings.swift]
ffi_module_filename = "auth_ffi"

[crates.suggest.bindings.swift]
ffi_module_filename = "suggest_ffi"

# Where to find our crates without cargo-metadata.
[crate-roots]
suggest = "./components/suggest"
```

and in `./components/suggest`, using `uniffi.toml` to [rename a Swift item](./renaming.md)

```toml
[bindings.swift.rename]
MyRecord = "SwiftRecord"
```

you'd use something like

```
uniffi-bindgen generate --config global.toml -l swift ...
```
