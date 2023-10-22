# Foreign traits

UniFFI traits can be implemented by foreign code.
This means traits implemented in Python/Swift/Kotlin etc can provide Rust code with capabilities not easily implemented in Rust, such as:

 * device APIs not directly available to Rust.
 * provide glue to clip together Rust components at runtime.
 * access shared resources and assets bundled with the app.

# Example

To implement a Rust trait in a foreign language, you might:

## 1. Define a Rust trait

This toy example defines a way of Rust accessing a key-value store exposed
by the host operating system (e.g. the key chain).

All methods of the Rust trait should return a `Result<>` with the error half being
a [compatible error type](./udl/errors.md) - see below for more on error handling.

For example:

```rust,no_run
#[uniffi::trait_interface]
pub trait Keychain: Send + Sync + Debug {
  fn get(&self, key: String) -> Result<Option<String>, KeyChainError>;
  fn put(&self, key: String, value: String) -> Result<(), KeyChainError>;
}
```

If you are using macros add `#[uniffi::export]` above the trait.
Otherwise define this trait in your UDL file:

```webidl
[Trait]
interface Keychain {
    [Throws=KeyChainError]
    string? get(string key);

    [Throws=KeyChainError]
    void put(string key, string data);
};
```

## 2. Allow it to be passed into Rust

Here, we define a new object with a constructor which takes a keychain.

```webidl
interface Authenticator {
    constructor(Keychain keychain);
    void login();
};
```

In Rust we'd write:

```rust,no_run
struct Authenticator {
  keychain: Arc<dyn Keychain>,
}

impl Authenticator {
  pub fn new(keychain: Arc<dyn Keychain>) -> Self {
    Self { keychain }
  }

  pub fn login(&self) {
    let username = self.keychain.get("username".into());
    let password = self.keychain.get("password".into());
  }
}
```

## 3. Create a foreign language implementation of the trait

Here's a Kotlin implementation:

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

## 4. Pass the implementation to Rust

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

Care is taken to ensure that things are cleaned up in the foreign language once all Rust references drop.

## ⚠️  Avoid cycles

Foreign trait implementations make it easy to create cycles between Rust and foreign objects causing memory leaks.
For example a foreign implementation holding a reference to a Rust object which also holds a reference to the same foreign implementation.

UniFFI doesn't try to help here and there's no universal advice; take the usual precautions.

# Error handling

We must handle foreign code failing, so all methods of the Rust trait should return a `Result<>` with a [compatible error type](./udl/errors.md) otherwise these errors will panic.

## Unexpected Error handling.

So long as your function returns a `Result<>`, it's possible for you to define how "unexpected" errors
(ie, errors not directly covered by your `Result<>` type, panics, etc) are converted to your `Result<>`'s `Err`.

If your code defines a `From<uniffi::UnexpectedUniFFICallbackError>` impl for your error type, then those errors will be converted into your error type which will be returned to the Rust caller.
If your code does not define this implementation the generated code will panic.
In other words, you really should implement this!

See our [callbacks example](https://github.com/mozilla/uniffi-rs/tree/main/examples/callbacks) for more.

