# Rust scaffolding

## Rust scaffolding code

Now we generate some Rust helper code to make the `add` method available to foreign-language bindings.  

First, add `uniffi` to your crate dependencies; this is the runtime support code that powers UniFFI's serialization of data types across languages:

```toml
[dependencies]
uniffi = "0.8"
```

Important note: the `uniffi` version must be the same as the `uniffi-bindgen` command-line tool installed on your system.

Then let's add `uniffi_build` to your build dependencies: it generates the Rust scaffolding code that exposes our Rust functions as a C-compatible FFI layer.

```toml
[build-dependencies]
uniffi_build = "0.8"
```

Then create a `build.rs` file next to `Cargo.toml` that will use `uniffi_build`:

```rust
fn main() {
    uniffi_build::generate_scaffolding("./src/math.udl").unwrap();
}
```

**Note:** This is the equivalent of calling (and does it under the hood) `uniffi-bindgen scaffolding src/math.udl --out-dir <OUT_DIR>`.

Lastly, we include the generated scaffolding code in our `lib.rs`. If you've used the default build
settings then this can be done using a handy macro:

```rust
uniffi_macros::include_scaffolding!("math");
```

If you have generated the scaffolding in a custom location, use the standard `include!` macro
to include the generated file by name, like this:


```rust
include!(concat!(env!("OUT_DIR"), "/math.uniffi.rs"));
```

**Note:** The file name is always `<namespace>.uniffi.rs`.

Great! `add` is ready to see the outside world!

### Avoiding version mismatches between `uniffi` core and `uniffi-bindgen`

The process above has one significant problem - things start to fall apart if
the version of the `uniffi` core (ie, the version specified in `[dependencies]`)
and the version of `uniffi-bindgen` (ie, the version installed by
`cargo install`) get out of date - and often these problems are not detected
until runtime when the generated rust code actually runs.

The `uniffi_build` crate supports an alternative workflow via the
`builtin-bindgen` feature. If this feature is enabled, then the `uniffi_build`
crate takes a runtime dependency on the `uniffi_bindgen` crate - effectively
building and running the `uniffi-bindgen` tool as your crate is being compiled.
The `uniffi-bindgen` tool doesn't need to be installed if this feature is
enabled.

The downside of this is that it drives up the build time for your crate (as
`uniffi-bindgen` needs to be built as well), so it's not the default.

To enable this, the `[build-dependencies]` of your Cargo.toml might look like:
```toml
[build-dependencies]
uniffi_build = {version = "0.8", features = [ "builtin-bindgen" ]}
```
Your `build.rs` script and everything else should remain the same, but now
whatever version of `uniffi` is specified will be used to perform the (now
slightly slower) build.

### Rust scaffolding code from a local `uniffi`

**Note:** This section is only for people who want to make changes to `uniffi`
itself. If you just want to use `uniffi` as released you should ignore this
section.

The techniques above don't work very well when you are making changes to
`uniffi` itself and want to see how those changes impact your project - there's
no released version of `uniffi` you can reference.

To support this use-case, you can leverage Cargo's support for local
dependencies and the `builtin-bindgen` feature described above. You should:

* Change the `[dependencies]` section of your Cargo.toml to point to a local
  checkout of `uniffi` core.

* Change the `[build-dependencies]` section of your Cargo.toml to point to a
  local checkout of `uniffi_build` *and* enable the `builtin-bindgen` feature.

For example, you will probably end up with Cargo.toml looking something like:

```toml
[dependencies]
uniffi = { path = "path/to/uniffi-rs/uniffi }
...
[build-dependencies]
uniffi_build = { path = "path/to/uniffi-rs/uniffi_build, features=["builtin-bindgen"] }
```

Note that `path/to/uniffi-rs` should be the path to the root of the `uniffi`
source tree - ie, the 2 path specs above point to different sub-directories
under the `uniffi` root.
