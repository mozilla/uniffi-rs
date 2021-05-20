# Interfaces/Objects

Interfaces are represented in the Rust world as a struct with an `impl` block containing methods. In the Kotlin or Swift world, it's a class.

Because Objects are passed by reference and Dictionaries by value, in the UniFFI world it is impossible to be both an Object and a [Dictionary](./structs.md).

The following Rust code:

```rust
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
```

would be exposed using:

```idl
interface TodoList {
    constructor();
    void add_item(string todo);
    sequence<string> get_items();
};
```

By convention, the `constructor()` calls the Rust's `new()` method.

Conceptually, these `interface` objects are live Rust objects that have a proxy on the foreign language side; calling any methods on them, including a constructor or destructor results in the corresponding methods being call in Rust.

UniFFI will generate these proxies of live objects with an interface or protocol.

e.g. in Kotlin.

```kotlin
interface TodoListInterface {
    fun addItem(todo: String)
    fun getItems(): List<String>
}

class TodoList : TodoListInterface {
   // implementations to call the Rust code.
}
```

When working with these objects, it may be helpful to always pass the interface or protocol, but construct the concrete implementation.

e.g. in Swift

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
alternate named constructors to create object instances in different ways. Each such constructor
must be given an explicit name, provided in the UDL with the `[Name]` attribute like so:

```idl
interface TodoList {
    // The default constructor makes an empty list.
    constructor();
    // This alternate constructor makes a new TodoList from a list of string items.
    [Name=new_from_items]
    constructor(sequence<string> items)
    ...
```

For each alternate constructor, UniFFI will expose an appropriate static-method, class-method or similar
in the foreign language binding, and will connect it to the Rust method of the same name on the underlying
Rust struct.


## Concurrent Access

Since interfaces represent mutable data, UniFFI has to take extra care
to uphold Rust's safety guarantees around shared and mutable references.
The foreign-language code may attempt to operate on an interface instance
from multiple threads, and it's important that this not violate Rust's
assumption that there is at most a single mutable reference to a struct
at any point in time.

By default, UniFFI enforces this using runtime locking. Each interface instance
has an associated lock which is transparently acquired at the beginning of each
call to a method of that instance, and released once the method returns. This
approach is simple and safe, but it means that all method calls on an instance
are run in a strictly sequential fashion, limiting concurrency.

You can opt out of this protection by marking the interface as threadsafe:

```idl
[Threadsafe]
interface Counter {
    constructor();
    void increment();
    u64 get();
};
```

The UniFFI-generated code will allow concurrent method calls on threadsafe interfaces
without any locking.

For this to be safe, the underlying Rust struct must adhere to certain restrictions, and
UniFFI's generated Rust scaffolding will emit compile-time errors if it does not.

The Rust struct must not expose any methods that take `&mut self`. The following implementation
of the `Counter` interface will fail to compile because it relies on mutable references:

```rust
struct Counter {
    value: u64
}

impl Counter {
    fn new() -> Self {
        Self { value: 0 }
    }

    // No mutable references to self allowed in [Threadsafe] interfaces.
    fn increment(&mut self) {
        self.value = self.value + 1;
    }

    fn get(&self) -> u64 {
        self.value
    }
}
```

Implementations can instead use Rust's "interior mutability" pattern. However, they
must do so in a way that is both `Sync` and `Send`, since the foreign-language code
may operate on the instance from multiple threads. The following implementation of the
`Counter` interface will fail to compile because `RefCell` is not `Send`:

```rust
struct Counter {
    value: RefCell<u64>
}

impl Counter {
    fn new() -> Self {
        // `RefCell` is not `Sync`, so neither is `Counter`.
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
```

This version uses an `AtomicU64` for interior mutability, which is both `Sync` and
`Send` and hence will compile successfully:

```rust
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
```

You can read more about the technical details in the docs on the
[internal details of managing object references](../internals/object_references.md).