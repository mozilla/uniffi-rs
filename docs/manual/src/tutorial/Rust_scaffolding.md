# Rust scaffolding

As already noted UniFFI supports two methods of interface definitions: UDL files and proc macros.

If you use proc macros you can skip the next section, but if you use UDL files, you need to generate Rust "scaffolding" - code to implement what's described in the UDL.

## Rust scaffolding code for UDL

Crates using UDL need a `build.rs` file next to `Cargo.toml`. This uses `uniffi` to generate the Rust scaffolding code.

```rust
fn main() {
    uniffi::generate_scaffolding("src/math.udl").unwrap();
}
```

It will generate `<namespace>.uniffi.rs` under your `target` directory.

Lastly, we include the generated scaffolding code in our `lib.rs` using this handy macro:

```rust
uniffi::include_scaffolding!("math");
```

### Setup for crates using only proc macros

If you are only using proc macros, you can skip `build.rs` entirely!
All you need to do is add this to the top of `lib.rs`

```rust
uniffi::setup_scaffolding!();
```

NOTE: This function takes an optional parameter, the [`namespace`](../udl/namespace.md) used by the component.
If not specified, the crate name will be used as the namespace.

**⚠ Warning ⚠** Do not call both `uniffi::setup_scaffolding!()` and `uniffi::include_scaffolding!!()` in the same crate.

### Libraries that depend on UniFFI components

Suppose you want to create a shared library that includes one or more
components using UniFFI. The typical way to achieve this is to create a new
crate that depends on the component crates.  However, this can run into
[rust-lang#50007](https://github.com/rust-lang/rust/issues/50007).  Under
certain circumstances, the scaffolding functions that the component crates
export do not get re-exported by the dependent crate.

Use the `uniffi_reexport_scaffolding!` macro to work around this issue.  If your
library depends on `foo_component`, then add
`foo_component::uniffi_reexport_scaffolding!();` to your `lib.rs` file and
UniFFI will add workaround code that forces the functions to be re-exported.

Each scaffolding function contains a hash that's derived from the UDL file.
This avoids name collisions when combining multiple UniFFI components into
one library.
