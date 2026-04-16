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

## Foreign traits.

It's possible to export the trait as a "foreign" trait by using `#[uniffi::export(with_foreign)]`.
This allows for the trait to also be implemented by foreign bindings - meaning the bindings can implement
an object and store it in Records or Enums, or pass them into Rust functions etc.

```rust
#[uniffi::export(with_foreign)]
pub trait Person {
    fn name() -> String;
    fn age() -> u32;
}
```

See the [foreign trait documentation](../foreign_traits.md) documentation for more.

## Callback Interfaces

Callback interfaces are very similar to, and pre-date Foreign traits.
Due to backwards compatibility, they appear in all signatures and Record/Enum definitions as `Box<dyn TraitName>` instead of `Arc<>`.
This makes them less useful in general - because Foreign traits offer all the same capabilities but don't have this limitation,
you should consider callback interfaces soft deprecated and use Foreign Traits instead.

See [their documentation for more](../types/callback_interfaces.md)

However, if you really do want to use them from proc-macros, you would use `#[uniffi::export(callback_interface)]`

```rust
#[uniffi::export(callback_interface)]
pub trait Person {
    fn name() -> String;
    fn age() -> u32;
}
```
