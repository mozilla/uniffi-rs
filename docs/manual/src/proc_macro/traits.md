# Traits

Traits can be exported with the `uniffi::export()` macro.

```rust
#[uniffi::export]
pub trait Person {
    fn name() -> String;
    fn age() -> u32;
}
```

These traits can then be used in the type model in the same was as [objects](./interfaces.md), but as usual, with `dyn` - ie, as `Arc<dyn Person>` - but see below
for an important exception.

For traits implemented in Rust, these will appear to the foreign side exactly like an [objects](./interfaces.md).
If the trait is exported as shown above, only Rust implemented traits will be able to be used.

## Specifying what crosses the FFI boundary

Use the `rust` and `foreign` flags to describe which implementations are available in the FFI for this trait:

| Attribute | Meaning |
|-----------|---------|
| `#[uniffi::export]` | Default is Rust implementations only |
| `#[uniffi::export(rust)]` | Explicit Rust implementations only |
| `#[uniffi::export(rust, foreign)]` | Both Rust and foreign implementations |
| `#[uniffi::export(foreign)]` | Foreign implementations only |

## Foreign traits

If the `uniffi::export` specified `foreign`, the trait can be implemented by foreign bindings - meaning
the bindings can implement an object and it's able to be used with functions, Records and Enums etc.

```rust
#[uniffi::export(rust, foreign)]
pub trait Person {
    fn name() -> String;
    fn age() -> u32;
}
```

If you never expose a Rust implementation over the FFI, using `#[uniffi::export(foreign)]` will avoid unused Rust scaffolding code.

See the [foreign trait documentation](../foreign_traits.md) documentation for more.

## Callback Interfaces (deprecated)

Callback interfaces are very similar to, and pre-date general trait support.
Due to backwards compatibility, they appear in all signatures and Record/Enum definitions as `Box<dyn TraitName>` instead of `Arc<>`.
This makes them less useful in general - because Foreign traits offer all the same capabilities but don't have this limitation,
you should consider callback interfaces soft deprecated and use Foreign Traits instead.
It's possible (but not planned) that callback interfaces will get in the way of some future work and be removed.

See [their documentation for more](../types/callback_interfaces.md)

If you really do want to use them from proc-macros, use `#[uniffi::export(callback_interface)]`:

```rust
#[uniffi::export(callback_interface)]
pub trait Person {
    fn name() -> String;
    fn age() -> u32;
}
```

## Deprecated syntax

The `with_foreign` flag is a deprecated alias for `rust, foreign` but still accepted for
backwards compatibility. We might remove it later, please use the new syntax.
