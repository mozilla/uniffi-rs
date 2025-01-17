# Configuration

The generated Kotlin modules can be configured using a `uniffi.toml` configuration file.

## Available options

| Configuration name           | Default                  | Description |
|------------------------------|--------------------------|------------ |
| `package_name`               | `uniffi`                 | The Kotlin package name - ie, the value used in the `package` statement at the top of generated files. |
| `cdylib_name`                | `uniffi_{namespace}`[^1] | The name of the compiled Rust library containing the FFI implementation (not needed when using `generate --library`). |
| `generate_immutable_records` | `false`                  | Whether to generate records with immutable fields (`val` instead of `var`). |
| `custom_types`               |                          | A map which controls how custom types are exposed to Kotlin. See the [custom types section of the manual](../types/custom_types.md#custom-types-in-the-bindings-code)|
| `external_packages`          |                          | A map of packages to be used for the specified external crates. The key is the Rust crate name, the value is the Kotlin package which will be used referring to types in that crate. See the [external types section of the manual](../types/remote_ext_types.md#kotlin)
| `android`                    | `false`                  | Used to toggle on Android specific optimizations
| `android_cleaner`            | `android`                | Use the [`android.system.SystemCleaner`](https://developer.android.com/reference/android/system/SystemCleaner) instead of [`java.lang.ref.Cleaner`](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/ref/Cleaner.html). Fallback in both instances is the one shipped with JNA.
| `kotlin_target_version`      | `"x.y.z"`                | When provided, it will enable features in the bindings supported for this version. The build process will fail if an invalid format is used.
| `omit_checksums`             | `false`                  | Whether to omit checking the library checksums as the library is initialized. Changing this will shoot yourself in the foot if you mixup your build pipeline in any way, but might speed up initialization.

## Example

Custom types
```toml
# Assuming a Custom Type named URL using a String as the builtin.
[bindings.kotlin.custom_types.Url]
# Name of the type in the Kotlin code
type_name = "URL"
# Classes that need to be imported
imports = [ "java.net.URI", "java.net.URL" ]
# Functions to convert between strings and URLs
into_custom = "URI({}).toURL()"
from_custom = "{}.toString()"
```

External types
```toml
[bindings.kotlin.external_packages]
# This specifies that external types from the crate `rust-crate-name` will be referred by by the package `"kotlin.package.name`.
rust-crate-name = "kotlin.package.name"
```

