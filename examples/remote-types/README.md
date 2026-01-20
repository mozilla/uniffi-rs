# Remote types example

This crate uses 2 types from remote crates: `log::Level` and `anyhow::Error`.
Remote types require special handling, since there's no way to add a `#[derive]` to a type that's
already been defined upstream.  Instead, use `#[uniffi::remote(<Kind>)]` to wrap the item's
definition.  UniFFI will generate the code to handle that type in the FFI, similar to if it had been
wrapped with `#[derive(uniffi::<Kind>)]`.

See https://mozilla.github.io/uniffi-rs/latest/types/remote_ext_types.html for more details.
