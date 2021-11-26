# Prerequisites

## The uniffi-bindgen cli tool

Install the `uniffi-bindgen` binary on your system using:

`cargo install uniffi_bindgen`

You can see what it can do with `uniffi-bindgen --help`, but let's leave it aside for now.

### Running from a source checkout

It's also possible to run `uniffi-bindgen` from a source checkout of uniffi - this might
be useful if you are experimenting with changes to uniffi and want to test them out.

In this case, just use `cargo run` in the `uniffi_bindgen` crate directory.
For example, from the root of the `uniffi-rs` repo, execute:

```shell
% cd uniffi_bindgen/src
% cargo run -- --help
```

and you will see the help output from running `uniffi-bindgen` locally. Refer to
the docs for `cargo run` for more information and options.

## Build your crate as a cdylib

Ensure your crate builds as a `cdylib` by adding
```toml
crate-type = ["cdylib"]
name = "<library name>"
```
to your crate's `Cargo.toml`.
