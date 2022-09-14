# Callback interfaces

Callback interfaces are traits specified in UDL which can be implemented by foreign languages.

They can provide Rust code access available to the host language, but not easily replicated
in Rust.

 * accessing device APIs
 * provide glue to clip together Rust components at runtime.
 * access shared resources and assets bundled with the app.

# Using callback interfaces

## 1. Define a Rust trait.

This toy example defines a way of Rust accessing a key-value store exposed
by the host operating system (e.g. the key chain).

```rust,no_run
trait Keychain: Send + Sync + Debug {
  pub fn get(key: String) -> Result<Option<String>, KeyChainError>
  pub fn put(key: String, value: String) -> Result<(), KeyChainError>
}
```

### Why + Send + Sync?

The concrete types that UniFFI generates for callback interfaces implement `Send`, `Sync`, and `Debug`, so it's safe to
include these as supertraits of your callback interface trait.  This isn't strictly necessary, but it's often useful.  In
particular, `Sync + Send` is useful when:
  - Storing `Box<dyn CallbackInterfaceTrait>` types inside a type that needs to be `Send + Send` (for example a UniFFI
    interface type)
  - Storing `Box<dyn CallbackInterfaceTrait>` inside a global `Mutex` or `RwLock`


## 2. Setup error handling

All methods of the Rust trait should return a Result.  The error half of that result must
be an [error type defined in the UDL](./errors.md).

In addition to expected errors, a callback interface call can result in all kinds of
unexpected errors.  Some examples are the foreign code throws an exception that's not part
of the exception type or there was a problem marshalling the data for the call.  UniFFI
uses `uniffi::UnexpectedUniFFICallbackError` for these cases.  Your code must include a
`From<uniffi::UnexpectedUniFFICallbackError>` impl for your error type to handle those.

It's currently allowed for callback interface methods to return a regular value rather
than a `Result<>`.  However, this is deprecated and we plan to remove support at some
point.

## 3. Define a callback interface in the UDL.

```webidl
callback interface Keychain {
    [Throws=KeyChainError]
    string? get(string key);

    [Throws=KeyChainError]
    void put(string key, string data);
};
```

## 4. And allow it to be passed into Rust.

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

## 4. Create an foreign language implementation of the callback interface.

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

## 5. Pass the implementation to Rust.

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
