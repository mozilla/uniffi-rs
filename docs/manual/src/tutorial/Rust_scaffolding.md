# Rust scaffolding

## Rust scaffolding code

Now we generate some Rust helper code to make the `add` method available to foreign-language bindings.  

First, add `uniffi` to your crate as both a dependency and build-dependency.  This adds the runtime support code that powers UniFFI and build-time support for generating the Rust scaffolding code.

```toml
[dependencies]
uniffi = "1.0.0"

[build-dependencies]
uniffi = "1.0.0"
```

Then create a `build.rs` file next to `Cargo.toml` that uses `uniffi` to generate the Rust scaffolding code.

```rust
fn main() {
    uniffi::generate_scaffolding("./src/math.udl").unwrap();
}
```

Lastly, we include the generated scaffolding code in our `lib.rs` using this handy macro:

```rust
uniffi::include_scaffolding!("math");
```

**Note:** The file name is always `<namespace>.uniffi.rs`.

Great! `add` is ready to see the outside world!

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
