# WASM

WASM defines a portable binary format for virtual machines running untrusted code in browsers and various other scenarios. Depending on the scenario, the host language may vary.

As such, `uniffi-rs` does not come with bindings generators "for WASM". This is the job for external uniffi-bindgen generators. e.g. [Typescript][ubrn], [Kotlin][gobley].

However, the generated scaffolding needs to be configured for that WASM32 target.

[ubrn]: https://github.com/jhugman/uniffi-bindgen-react-native
[gobley]: https://github.com/gobley/gobley

# Configuration

Configuring the Rust scaffolding ready for WASM binding generators is done by opting into features.

e.g.

```toml
[dependendencies]
uniffi = { version = "0.29.2", features = ["wasm-unstable-single-threaded"]}
```

## Features

### `wasm-unstable-single-threaded`

At time of writing, the state of threading in WASM and Rust support is still in flux.

Much of the ecosystem is built around [gloo][gloo], [`web-sys`][websys] and [`wasm-rayon`][rayon].

Enabling the `wasm-unstable-single-threaded` feature opts out of the `Send` and `Sync` checks when building for `wasm32` target architectures.

This feature only affects the `wasm32` architecture.

Hint: running `clippy` on client code built for `wasm32` may trigger a clippy [warning when constructing `Arc`s][clippy/arc].

The lint's remedy is either to:
- make the object to `Send` and `Sync`, or
- change from an `Arc` to an `Rc`.

As we are using this feature:
- to allow `Send` and `Sync`, and
- we can't use `Rc` for other targets

we can disable this warning for `wasm32` *only* like so:

```rust
#[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
fn new() -> Arc<Self> {
    Arc::new(Self {})
}
```

Note: as support for WASM threads evolves, this feature is likely to change or go away completely.

[gloo]: https://crates.io/crates/gloo
[webdev/thread]: https://web.dev/articles/webassembly-threads#rust
[rayon]: https://github.com/RReverser/wasm-bindgen-rayon
[websys]: https://docs.rs/web-sys/latest/web_sys/
[clippy/arc]: https://rust-lang.github.io/rust-clippy/master/index.html#arc_with_non_send_sync
