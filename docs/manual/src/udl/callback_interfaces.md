# Callback interfaces

Callback interfaces are traits specified in UDL which can be implemented by
foreign languages.  Callbacks interfaces allow the library consumer to supply
the Rust library with additional capabilities implemented in the foreign
language, for example:

 * accessing app preferences and assets
 * network requests
 * logging
 * task scheduling

Both sync and async functions are supported.

# Defining callback interfaces

```rust,no_run
#[uniffi::callback_interface]
pub trait Keychain {
  // Access a keychain exposed by the host operating system
  fn get(&self, key: String) -> Result<Option<String>, KeyChainError>
  fn put(&self, key: String, value: String) -> Result<(), KeyChainError>

  // Async API
  async fn get_async(&self, key: String) -> Result<Option<String>, KeyChainError>
  async fn put_async(&self, key: String, value: String) -> Result<(), KeyChainError>
}
```

Note: Rust doesn't support async functions in traits yet.  The proc-macro
converts async function signatures into a sync function that returns
`uniffi::ForeignFuture<T>`, similar to the `async-trait` crate.
`uniffi::ForeignFuture<T>` implements `Future<Output = T>` and `Send`.

## Setup error handling

All methods of the Rust trait should return a Result.  This is needed because
any callback method call can fail unexpected ways that shouldn't result in a
panic, for example when the foreign code throws an exception we weren't
expecting.

If your method can only fail in these unexpected ways, use
`uniffi::CallbackResult<T>` as your result type.  This is a type alias for
`Result<T, uniffi::UnexpectedCallbackError>`.

For methods that are expected to throw sometimes, use [error type defined in the UDL](./errors.md)
and implement `From<uniffi::UnexpectedCallbackError>` for that type. See
`example/callbacks` for an example of how to do this.

# Using callback interfaces

## Create an foreign language implementation of the callback interface

In this example, here's a Kotlin implementation.

```kotlin
class KotlinKeychain: Keychain {
    override fun get(key: String): String? {
        // … elide the implementation.
        return value
    }
    override fun put(key: String, value: String) {
        // … elide the implementation.
    }

    override async fun getAsync(key: String) = withContext(coroutineContext) {
        get(key)
    }

    override async fun putAsync(key: String, value: String) = withContext(coroutineContext) {
        put(key, value)
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
    func put(key: String, value: String) {
        // … elide the implementation.
    }

    func getAsync(key: String) async -> String? {
        // call get() in a background thread and return the value
    }
    func putAsync(key: String, value: String) async {
        // call put() in a background thread
    }
}
```

Note: in Swift, this must be a `class`.

## Pass the callback interface into Rust

Here, we define a constructor to pass the keychain to rust, then methods that use it.

In UDL:

```webidl
interface Authenticator {
    constructor(Keychain keychain);
    UserInfo get_password();
    async UserInfo get_password_async();
}
```

In Rust:

```rust,no_run
struct Authenticator {
  keychain: Keychain,
}

impl Authenticator {
  pub fn new(keychain: Keychain) -> Self {
    Self { keychain }
  }
  pub fn get_password(&self) -> String {
    UserInfo {
        username: self.keychain.get("username".into()),
        password: self.keychain.get("password".into()),
    }
  }

  pub async fn get_password_async(&self) -> String {
    UserInfo {
      username: await self.keychain.get_async("username".into())
      password: await self.keychain.get_async("password".into())
    }
  }
}
```

Note how we can define async Rust methods based on async callback interface methods.  Foreign code
can await `get_password_async()` and resume when the two `get_async()` futures complete.  This is
all driven by the foreign executor/event loop, there's no need to start a tokio loop.

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

## Scheduling tasks using the `TaskQueue` callback interface

UniFFI defines the a builtin callback interface, `TaskQueue`,  which can be used to run code using
the foreign language executor (Kotlin CoroutineScope, Python Executor, Swift DispatchQueue, etc.).

You can then use the `TaskQueue::schedule()` method to run the closure in that background task queue
and return a future for the result of the closure.

```rust,no_run
pub struct UserStore {
    connection: DatabaseConnection,
    background_queue: uniffi::TaskQueue,
}

impl UserStore {
    pub fn new(background_queue: uniffi::TaskQueue) -> Self {
        Self {
            connection: DatabaseConnection::new(),
            background_queue,
        }
    }

    pub async fn add_user(&self, username: String, password: String) {
        await self.background_queue.schedule(move || {
            self.connection.execute(format!("INSERT INTO users(username, password) VALUES ({username}, {password})"));
        }
    }

    pub async fn lookup_password(&self, username: String) -> String {
        await self.background_queue.schedule(move || {
            self.connection.execute(format!("SELECT password FROM users WHERE username = {username}"))
        }
    }

    pub async fn delete_user(&self, username: String) -> String {
        await self.background_queue.schedule(move || {
            self.connection.execute(format!("DELETE FROM users WHERE username = "{username}"));
        }
    }
```

You can also use the `schedule_later<T>(delay: Duration, closure: FnOnce() -> T) -> T` method to run the closure after a delay.

Task queues can be created on the foreign bindings side by passing in the native executer, for example:

```kotlin
val userStore = UserStore(TaskQueue(Dispatchers.IO))
```

## Async callbacks are driven by foreign executors

Code that awaits an async callback interface method will be woken up by an FFI call from the UniFFI
foreign bindings code. This allows your Rust code to be driven by the foreign async executor,
avoiding the need to spawn threads from Rust or run a `tokio` loop.

See the `user-data-store` example for how this could work in action.

## Fire and forget callbacks

As mentioned above, UniFFI uses the `ForeignFuture<T>` type to handle async callback interface
returns.  `ForeignFuture` also implements the `fire()` method, which schedules the future to be run
by the foreign executor, ignoring the result.  This is good for fire-and-forget style callbacks,
like logging.

## Combining tokio with a foreign executor

... TODO, I think this should be possible but have no clue how it would work
