# Objects

Objects are represented in the Rust world as a struct with an `impl` block containing methods.
In the foreign-language bindings they would typically correspond to a class.

In the interface definition, objects may have public methods but *must not have public fields*,
because they are intended to be opaque to the foreign-language code. Like this:

```rust
#[uniffi_macros::declare_interface]
mod example {
    struct TodoList {
        items: Vec<String>
    }

    impl TodoList {
        fn new() -> Self {
            TodoList {
                items: Vec::new()
            }
        }

        fn add_item(&mut self, todo: String) {
            self.items.push(todo);
        }

        fn get_items(&self) -> Vec<String> {
            self.items.clone()
        }
    }
}
```

Conceptually, these objects are live Rust objects that have a proxy on the foreign language side; calling any methods on them, including a constructor or destructor results in the corresponding methods being call in Rust.

UniFFI will generate these proxies of live objects with an interface or protocol. For example,
in Kotlin this interface would produce:


```kotlin
interface TodoListInterface {
    fun addItem(todo: String)
    fun getItems(): List<String>
}

class TodoList : TodoListInterface {
   // implementations to call the Rust code.
}
```

When working with these objects, it may be helpful to always pass the interface or protocol, but construct the concrete implementation. For example, in Swift:

```swift
let todoList = TodoList()
todoList.addItem(todo: "Write documentation")
display(list: todoList)

func display(list: TodoListProtocol) {
    let items = list.getItems()
    items.forEach {
        print($0)
    }
}
```

## Alternate Named Constructors

In addition to the default constructor connected to the `::new()` method, you can specify
alternate named constructors to create object instances in different ways. These are
declared in the Rust code as methods without a `self` argument and returning an owned
instance of `Self`:

```rust
    impl TodoList {
        // The default constructor makes an empty list.
        fn new() -> Self {
            TodoList {
                items: Vec::new()
            }
        }

        // This alternate constructor makes a new TodoList from a list of string items.
        fn new_from_items(items: Vec<String>) -> Self {
            let mut obj = Self::new();
            for item in items {
                obj.add_item(item);
            }
            obj
        }
    }
```

For each alternate constructor, UniFFI will expose an appropriate static-method, class-method or similar
in the foreign language binding, and will connect it to the Rust method of the same name on the underlying Rust struct.


## Concurrent Access

Since interfaces represent mutable data, UniFFI has to take extra care
to uphold Rust's safety guarantees around shared and mutable references.
The foreign-language code may attempt to operate on an interface instance
from multiple threads, and it's important that this not violate Rust's
assumption that there is at most a single mutable reference to a struct
at any point in time.

UniFFI uses the public interface of your struct to determine how to
enforce this:

* If any exposed instance method uses `&mut self` as its receiver argument,
  UniFFI will wrap each instance in a mutex that is transparently acquired
  for the duration of each method call, ensuring safe concurrent access
  but limiting potential concurrency.
* If all exposed instance methods use `&self` as their receiver argument,
  then UniFFI will insist that your struct be `Sync` and `Send` and will
  allow direct concurrent calls from the foreign-language bindings.

(Note that methods taking an owned `self` reciver argument are not supported).

As a small example, suppose we wanted to expose a `Counter` class from Rust
to foreign-language code. This definition would work, but each method call
would involve taking a Mutex and thus would limit concurrency:

```rust
#[uniffi_macros::declare_interface]
mod example {
    struct Counter {
        value: u64
    }

    impl Counter {
        fn new() -> Self {
            Self { value: 0 }
        }

        // `&mut self` means UniFFI will add locking to ensure safety.
        fn increment(&mut self) {
            self.value = self.value + 1;
        }

        fn get(&self) -> u64 {
            self.value
        }
    }
}
```

Implementations can instead use Rust's "interior mutability" pattern to avoid `&mut self`
and allow concurrent access. However, they must do so in a way that is both `Sync` and `Send`,
since the foreign-language code may operate on the instance from multiple threads. The following implementation of the `Counter` interface will fail to compile because `RefCell` is not `Send`:

```rust
#[uniffi_macros::declare_interface]
mod example {
    // Error: `Counter` is not `Sync` because `RefCell` is not `Sync`,
    // so UniFFI will generate a compile-time error.
    struct Counter {
        value: RefCell<u64>
    }

    impl Counter {
        fn new() -> Self {
            Self { value: RefCell::new(0) }
        }

        fn increment(&self) {
            let mut value = self.value.borrow_mut();
            *value = *value + 1;
        }

        fn get(&self) -> u64 {
            *self.value.borrow()
        }
    }
}
```

This version uses an `AtomicU64` for interior mutability, which is both `Sync` and
`Send` and hence will compile successfully:

```rust
#[uniffi_macros::declare_interface]
mod example {
    struct Counter {
        value: AtomicU64
    }

    impl Counter {
        fn new() -> Self {
            Self { value: AtomicU64::new(0) }
        }

        fn increment(&self) {
            self.value.fetch_add(1, Ordering::SeqCst);
        }

        fn get(&self) -> u64 {
            self.value.load(Ordering::SeqCst)
        }
    }
}
```

You can read more about the technical details in the docs on the
[internal details of managing object references](../internals/object_references.md).
