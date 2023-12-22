# Callback interfaces

Callback interfaces are a special implementation of
[Rust traits implemented by foreign languages](../foreign_traits.md).

These are described in both UDL and proc-macros as an explicit "callback interface".
They are (soft) deprecated, remain now for backwards compatibility, but probably
should be avoided.

This document describes the differences from regular traits.

## Defining a callback
If you must define a callback in UDL it would look like:
```webidl
callback interface Keychain {
  // as described in the foreign traits docs...
};
```

procmacros support it too, but just don't use it :)

### Box and Send + Sync?

Traits defined purely for callbacks probably don't technically need to be `Sync` in Rust, but
they conceptually are, just outside of Rust's view.

That is, the methods of the foreign class must be safe to call
from multiple threads at once, but Rust can not enforce this in the foreign code.

## Rust signature differences

Consider the examples in [Rust traits implemented by foreign languages](../foreign_traits.md).

If the traits in question are defined as a "callback" interface, the `Arc<dyn Keychain>` types
would actually be `Box<dyn Keychain>` - eg, the Rust implementation of the `Authenticator`
constructor would be ```fn new(keychain: Box<dyn Keychain>) -> Self``` instead of the `Arc<>`.
