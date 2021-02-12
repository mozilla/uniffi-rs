# Objects

Objects are represented in the Rust world as a struct with an `impl` block containing methods.
In the foreign-language bindings they would typically correspond to a class.

In the interface definition, objects may have public methods but *must not have public fields*,
because they are intended to be opaque to the foreign-language code. Like this:

```rust
#[uniffi::declare_interface]
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

Conceptually, these objects are live Rust objects that have a proxy on the foreign language side; calling any methods on them, including a constructor or destructor results in the corresponding methods being called in Rust.

UniFFI will generate these proxies of live objects with an interface or protocol. For example,
in Kotlin the `TodoList` object would produce:


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


## Managing Shared References

To the foreign-language consumer, UniFFI object instances are designed to behave as much like
regular language objects as possible. They can be freely passed as arguments or returned as values,
like this Kotlin API:

```kotlin
interface TodoListInterface {
    // Copy the items from another TodoList into this one.
    // TODO: hrm, actually we need to pass a concrete `TodoList` instance here,
    // not just anything that implements the interface...
    void importItems(TodoListInterface other);

    // Make a copy of this TodoList as a new instance.
    TodoListInterface duplicate();
}
```

To ensure that this is safe, UniFFI allocates every object instance on the heap using
[`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html), Rust's built-in smart pointer
type for managing shared references at runtime.

The use of `Arc` is transparent to the foreign-language code, but sometimes shows up
in the function signatures of the underlying Rust code. For example, the Rust code implementing
the `TodoList::duplicate` method would need to explicitly return an `Arc<TodoList>`, since UniFFI
doesn't know whether it will be returning a new object or an existing one:

```rust
impl TodoList {
    fn duplicate(&self) -> Arc<TodoList> {
        Arc::new(TodoList {
            items: RwLock::new(self.items.read().unwrap().clone())
        })
    }
}
```

Object instances passed as function arguments may be received as an `Arc<T>` if the underlying
code requires a strong reference:

```rust
impl TodoList {
    fn import_items(&self, other: Arc<TodoList>) {
        self.items.write().unwrap().append(other.get_items());
    }
}
```

But UniFFI is smart enough to borrow from the `Arc` if the underlying Rust method only
needs an `&T`, so this would also work:

```rust
impl TodoList {
    //                              +-- don't need to care about the `Arc` here
    //                              V
    fn import_items(&self, other: &TodoList) {
        self.items.write().unwrap().append(other.get_items());
    }
}
```

The same works for the implicit `self` parameter, which may be explicitly specified
to take an owned `Arc` if needed:


```rust
impl TodoList {
    // `Arc`s everywhere! --+-----------------+
    //                      V                 V
    fn import_items(self: Arc<Self>, other: Arc<TodoList>) {
        self.items.write().unwrap().append(other.get_items());
    }
}
```

You can read more about the technical details in the docs on the
[internal details of managing object references](../internals/object_references.md).


## Concurrent Access

Since objects represent mutable data, UniFFI has to take extra care
to uphold Rust's safety guarantees around shared and mutable references.
The foreign-language code may attempt to operate on an object instance
from multiple threads, and it's important that this not violate Rust's
assumption that there is at most a single mutable reference to a struct
at any point in time.

UniFFI enforces this by requiring that the Rust implementation of an object
be `Sync+Send`, and you will get a compile-time error if your implementation
does not satisfy this requirement.

As a small example, suppose we wanted to expose a `Counter` class from Rust
to foreign-language code. The following implementation will fail to compile because its
public methods take mutable references to `&self` and are therefore not safe to call
concurrently:

```rust
#[uniffi::declare_interface]
pub mod counter {
    struct Counter {
        value: u64
    }

    impl Counter {
        fn new() -> Self {
            Self { value: 0 }
        }

        // Compile-time error!
        //
        // No mutable references to self allowed in UniFFI interfaces.
        fn increment(&mut self) {
            self.value = self.value + 1;
        }

        fn get(&self) -> u64 {
            self.value
        }
    }
}
```


Implementations can instead use Rust's "interior mutability" pattern. However, they
must do so in a way that is both `Sync` and `Send`, since the foreign-language code
may operate on the instance from multiple threads. The following implementation of the
`Counter` interface will fail to compile because `RefCell` is not `Sync`:


```rust
#[uniffi::declare_interface]
pub mod counter {
    struct Counter {
        // Compile-time Error!
        //
        // `RefCell` is not `Sync`, so neither is `Counter`.
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
#[uniffi::declare_interface]
pub mod counter {
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
