# Callback interfaces

Callback interfaces are traits that can be implemented by foreign-language code
and used from Rust. They can help provide Rust code with access to features
available to the host language, but not easily replicated in Rust:

 * accessing device APIs
 * provide glue to clip together Rust components at runtime.
 * access shared resources and assets bundled with the app.

Callback interfaces are defined in the interface definition as a `pub trait`
with one or more methods. This toy example defines a way of Rust accessing
a key-value store exposed by the host operating system (e.g. the system keychain).

```rust
trait Keychain: Send {
  pub fn get(key: String) -> Option<String>
  pub fn put(key: String, value: String)
}
```

In order to actually *use* a callback interface, the Rust code must also
provide some other function or method that accepts it as an argument,
like this:

```rust
pub struct Authenticator {
  keychain: Box<dyn Keychain>,
}

impl Authenticator {
  pub fn new(keychain: Box<dyn Keychain>) -> Self {
    Self { keychain }
  }
  pub fn login(&self) {
    let username = self.keychain.get("username".into());
    let password = self.keychain.get("password".into());
    // Go ahead and use the credentials...
  }
}
```

You can then create a foreign language implementation of the callback interface;
here's an example in Kotlin:

```kotlin
class AndroidKeychain: Keychain {
    override fun get(key: String): String? {
        // … elide the implementation.
        return value
    }
    override fun put(key: String) {
        // … elide the implementation.
    }
}
```

And pass the implementation to Rust:

```kotlin
val authenticator = Authenticator(AndroidKeychain())
// later on:
authenticator.login()
```

The generated bindings take care to ensure that once the `Box<dyn Keychain>` is dropped in Rust,
then it is cleaned up in Kotlin.

Also note, that storing the `Box<dyn Keychain>` in the `Authenticator` required that all implementations
*must* implement `Send`.