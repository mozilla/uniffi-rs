# Callback interfaces

Callback interfaces are traits specified in UDL which can be implemented by foreign languages.

They can provide Rust code access available to the host language, but not easily replicated
in Rust.

 * accessing device APIs
 * provide glue to clip together Rust components at runtime.
 * access shared resources and assets bundled with the app.

# Using callback interfaces

1. Define a Rust trait. 

This toy example defines a way of Rust accessing a key-value store exposed
by the host operating system (e.g. the key chain).

```
trait Keychain: Send {
  pub fn get(key: String) -> Option<String>
  pub fn put(key: String, value: String)
}
```

2. Define a callback interface in the UDL

```
callback interface Keychain {
    string? get(string key);
    void put(string key, string data);
};
```

3. And allow it to be passed into Rust.

Here, we define a constructor to pass the keychain to rust, and then another method
which may use it.

In UDL:
```
object Authenticator {
    constructor(Keychain keychain);
    void login();
}
```

In Rust:

```
struct Authenticator {
  keychain: Box<dyn Keychain>,
}

impl Authenticator {
  pub fn new(keychain: Box<dyn Keychain>) -> Self {
    Self { keychain }
  }
  pub fn login() {
    let username = keychain.get("username".into());
    let password = keychain.get("password".into());
  }
}
```
4. Create an foreign language implementation of the callback interface.

In this example, here's a Kotlin implementation.

```
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
5. Pass the implementation to Rust.

Again, in Kotlin

```
val authenticator = Authenticator(AndroidKeychain())
// later on:
authenticator.login()
```

Care is taken to ensure that once `Box<dyn Keychain>` is dropped in Rust, then it is cleaned up in Kotlin.

Also note, that storing the `Box<dyn Keychain>` in the `Authenticator` required that all implementations
*must* implement `Send`.