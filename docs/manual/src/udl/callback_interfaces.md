# Callback interfaces

Callback interfaces are traits specified in UDL which can be implemented by foreign languages.

They can provide Rust code access available to the host language, but not easily replicated
in Rust.

 * accessing device APIs.
 * provide glue to clip together Rust components at runtime.
 * access shared resources and assets bundled with the app.

# Using callback interfaces

## 1. Define a Rust trait

This toy example defines a way of Rust accessing a key-value store exposed
by the host operating system (e.g. the key chain).

```rust,no_run
pub trait Keychain: Send + Sync + Debug {
  fn get(&self, key: String) -> Result<Option<String>, KeyChainError>
  fn put(&self, key: String, value: String) -> Result<(), KeyChainError>
}
```

### Send + Sync?

The concrete types that UniFFI generates for callback interfaces implement `Send`, `Sync`, and `Debug`, so it's legal to
include these as supertraits of your callback interface trait.  This isn't strictly necessary, but it's often useful.  In
particular, `Send + Sync` is useful when:
  - Storing `Box<dyn CallbackInterfaceTrait>` types inside a type that needs to be `Send + Sync` (for example a UniFFI
    interface type)
  - Storing `Box<dyn CallbackInterfaceTrait>` inside a global `Mutex` or `RwLock`

**⚠ Warning ⚠**: this comes with a caveat: the methods of the foreign class must be safe to call
from multiple threads at once, for example because they are synchronized with a mutex, or use
thread-safe data structures.  However, Rust can not enforce this in the foreign code.  If you add
`Send + Sync` to your callback interface, you must make sure to inform your library consumers that
their implementations must logically be Send + Sync.

## 2. Setup error handling

All methods of the Rust trait should return a Result.  The error half of that result must
be an [error type defined in the UDL](./errors.md).

It's currently allowed for callback interface methods to return a regular value
rather than a `Result<>`.  However, this is means that any exception from the
foreign bindings will lead to a panic.

### Extra requirements for errors used in callback interfaces

In order to support errors in callback interfaces, UniFFI must be able to
properly [lift the error](../internals/lifting_and_lowering.md).  This means
that the if the error is described by an `enum` rather than an `interface` in
the UDL (see [Errors](./errors.md)) then all variants of the Rust enum must be unit variants.

In addition to expected errors, a callback interface call can result in all kinds of
unexpected errors.  Some examples are the foreign code throws an exception that's not part
of the exception type or there was a problem marshalling the data for the call.  UniFFI
uses `uniffi::UnexpectedUniFFICallbackError` for these cases.  Your code must include a
`From<uniffi::UnexpectedUniFFICallbackError>` impl for your error type to handle those or
the UniFFI scaffolding code will fail to compile.  See `example/callbacks` for an
example of how to do this.

## 3. Define a callback interface in the UDL

```webidl
callback interface Keychain {
    [Throws=KeyChainError]
    string? get(string key);

    [Throws=KeyChainError]
    void put(string key, string data);
};
```

## 4. And allow it to be passed into Rust

Here, we define a constructor to pass the keychain to rust, and then another method
which may use it.

In UDL:

```webidl
interface Authenticator {
    constructor(Keychain keychain);
    void login();
}
```

In Rust:

```rust,no_run
struct Authenticator {
  keychain: Box<dyn Keychain>,
}

impl Authenticator {
  pub fn new(keychain: Box<dyn Keychain>) -> Self {
    Self { keychain }
  }
  pub fn login(&self) {
    let username = self.keychain.get("username".into());
    let password = self.keychain.get("password".into());
  }
}
```

## 5. Create an foreign language implementation of the callback interface

In this example, here's a Kotlin implementation.

```kotlin
class KotlinKeychain: Keychain {
    override fun get(key: String): String? {
        // … elide the implementation.
        return value
    }
    override fun put(key: String) {
        // … elide the implementation.
    }
}
```

…and Swift:

```swift
class SwiftKeychain: Keychain {
    func get(key: String) -> String? {
        // … elide the implementation.
        return value
    }
    func put(key: String) {
        // … elide the implementation.
    }
}
```

Note: in Swift, this must be a `class`.

## 6. Pass the implementation to Rust

Again, in Kotlin

```kt
val authenticator = Authenticator(KotlinKeychain())
// later on:
authenticator.login()
```

and in Swift:

```swift
let authenticator = Authenticator(SwiftKeychain())
// later on:
authenticator.login()
```

Care is taken to ensure that once `Box<dyn Keychain>` is dropped in Rust, then it is cleaned up in the foreign language.

Also note, that storing the `Box<dyn Keychain>` in the `Authenticator` required that all implementations
*must* implement `Send`.

## ⚠️  Avoid callback interfaces cycles

Callback interfaces can create cycles between Rust and foreign objects and lead to memory leaks.  For example a callback
interface object holds a reference to a Rust object which also holds a reference to the same callback interface object.
Take care to avoid this by following guidelines:

1. Avoid references to UniFFI objects in callback objects, including direct references and transitive
   references through intermediate objects.
2. Avoid references to callback objects from UniFFI objects, including direct references and transitive
   references through intermediate objects.
3. If you need to break the first 2 guidelines, then take steps to manually break the cycles to avoid memory leaks.
   Alternatively, ensure that the number of objects that can ever be created is bounded below some acceptable level.
