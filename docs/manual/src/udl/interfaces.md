# Interfaces/Objects

Interfaces are represented in the Rust world as a struct with an `impl` block containing methods. In the Kotlin or Swift world, it's a class.

Because Objects are passed by reference and Dictionaries by value, in the UniFFI world it is impossible to be both an Object and a [Dictionary](./structs.md).

The following Rust code:

```rust
struct TodoList {
    items: RwLock<Vec<String>>
}

impl TodoList {
    fn new() -> Self {
        TodoList {
            items: RwLock::new(Vec::new())
        }
    }

    fn add_item(&self, todo: String) {
        self.items.write().unwrap().push(todo);
    }

    fn get_items(&self) -> Vec<String> {
        self.items.read().unwrap().clone()
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

Conceptually, these `interface` objects are live Rust structs that have a proxy object on the foreign language side; calling any methods on them, including a constructor or destructor results in the corresponding methods being called in Rust. If you do not specify a constructor the bindings will be unable to create the interface directly.

UniFFI will generate these proxies with an interface or protocol to help with testing in the foreign-language code. For example in Kotlin, the `TodoList` would generate:

```kotlin
interface TodoListInterface {
    fun addItem(todo: String)
    fun getItems(): List<String>
}

class TodoList : TodoListInterface {
   // implementations to call the Rust code.
}
```

When working with these objects, it may be helpful to always pass the interface or protocol, but construct the concrete implementation. For example in Swift:

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

Following this pattern will make it easier for you to provide mock implementation of the Rust-based objects
for testing.

## Exposing Traits as interfaces

It's possible to have UniFFI expose a Rust trait as an interface by specifying a `Trait` attribute.

For example, in the UDL file you might specify:

```idl
[Trait]
interface Button {
    string name();
};

```

With the following Rust implementation:

```rust
pub trait Button: Send + Sync {
    fn name(&self) -> String;
}

struct StopButton {}

impl Button for StopButton  {
    fn name(&self) -> String {
        "stop".to_string()
    }
}
```

Uniffi explicitly checks all interfaces are `Send + Sync` - there's a ui-test which demonstrates obscure rust compiler errors when it's not true. Traits however need to explicitly add those bindings.

References to traits are passed around like normal interface objects - in an `Arc<>`.
For example, this UDL:

```idl
namespace traits {
    sequence<Button> get_buttons();
    Button press(Button button);
};
```

would have these signatures in Rust:

```rust
fn get_buttons() -> Vec<Arc<dyn Button>> { ... }
fn press(button: Arc<dyn Button>) -> Arc<dyn Button> { ... }
```

See the ["traits" example](https://github.com/mozilla/uniffi-rs/tree/main/examples/traits) for more.

### Traits construction

Because any number of `struct`s may implement a trait, they don't have constructors.

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
    constructor(sequence<string> items);
    ...
```

For each alternate constructor, UniFFI will expose an appropriate static-method, class-method or similar
in the foreign language binding, and will connect it to the Rust method of the same name on the underlying
Rust struct.

## Exposing methods from standard Rust traits

Rust has a number of general purpose traits which add functionality to objects, such
as `Debug`, `Display`, etc. It's possible to tell UniFFI that your object implements these
traits and to generate FFI functions to expose them to consumers. Bindings may then optionally
generate special methods on the object.

For example, consider the following example:
```
[Traits=Debug]
interface TodoList {
    ...
}
```
and the following Rust code:
```rust
#[derive(Debug)]
struct TodoList {
   ...
}
```

This will cause the Python bindings to generate a `__repr__` method that returns the value implemented by the `Debug` trait.
Not all bindings support generating special methods, so they may be ignored.
It is your responsibility to implement the trait on your objects; UniFFI will attempt to generate a meaningful error if you do not.

The list of supported traits is hard-coded in UniFFI's internals, and at time of writing
is `Debug`, `Display`, `Eq` and `Hash`.

## Managing Shared References

To the foreign-language consumer, UniFFI object instances are designed to behave as much like
regular language objects as possible. They can be freely passed as arguments or returned as values,
like this:

```idl
interface TodoList {
    ...

    // Copy the items from another TodoList into this one.
    void import_items(TodoList other);

    // Make a copy of this TodoList as a new instance.
    TodoList duplicate();
};
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

By default, object instances passed as function arguments will also be passed as an `Arc<T>`, so the
Rust implementation of `TodoList::import_items` would also need to accept an `Arc<TodoList>`:

```rust
impl TodoList {
    fn import_items(&self, other: Arc<TodoList>) {
        self.items.write().unwrap().append(other.get_items());
    }
}
```

If the Rust code does not need an owned reference to the `Arc`, you can use the `[ByRef]` UDL attribute
to signal that a function accepts a borrowed reference:

```idl
interface TodoList {
    ...
    //                  +-- indicate that we only need to borrow the other list
    //                  V
    void import_items([ByRef] TodoList other);
    ...
};
```

```rust
impl TodoList {
    //                              +-- don't need to care about the `Arc` here
    //                              V
    fn import_items(&self, other: &TodoList) {
        self.items.write().unwrap().append(other.get_items());
    }
}
```

Conversely, if the Rust code explicitly *wants* to deal with an `Arc<T>` in the special case of
the `self` parameter, it can signal this using the `[Self=ByArc]` UDL attribute on the method:


```idl
interface TodoList {
    ...
    // +-- indicate that we want the `Arc` containing `self`
    // V
    [Self=ByArc]
    void import_items(TodoList other);
    ...
};
```

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

Since interfaces represent mutable data, UniFFI has to take extra care
to uphold Rust's safety guarantees around shared and mutable references.
The foreign-language code may attempt to operate on an interface instance
from multiple threads, and it's important that this not violate Rust's
assumption that there is at most a single mutable reference to a struct
at any point in time.

UniFFI enforces this by requiring that the Rust implementation of an interface
be `Sync+Send`, and you will get a compile-time error if your implementation
does not satisfy this requirement. For example, consider a small "counter"
object declared like so:

```idl
interface Counter {
    constructor();
    void increment();
    u64 get();
};
```

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

    // No mutable references to self allowed in UniFFI interfaces.
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
`Counter` interface will fail to compile because `RefCell` is not `Sync`:

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
