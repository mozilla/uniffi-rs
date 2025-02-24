# Interfaces, Objects and Traits

Interfaces can have constructors and have methods. In Rust, they are represented as `impl` blocks. In the Kotlin or Swift world, they are a class, so they are often known as "Objects"

Interfaces are passed by reference so can not have data items - unlike a [Record](./records.md) or [Enum](./enumerations.md), which are passed by value so *only* have data fields and no methods.

Interfaces must be exposed via [UDL](../udl/interfaces.md) or [proc-macros](../proc_macro/interfaces.md)

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

These `interface` objects are live Rust structs behind an `Arc<>` that have a proxy object on the foreign language side;
calling any methods on them, including a constructor results in the corresponding methods being called in Rust.

UniFFI will generate these proxies with an interface or protocol to help with testing in the foreign-language code. For example in Kotlin, the `TodoList` would generate:

```kotlin
interface TodoListInterface {
    fun addItem(todo: String)
    fun getItems(): List<String>
};

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


## Constructors

Interfaces can have one or more constructors. They must have a constructor to be directly created from foreign bindings.

`TodoList` has a `new()` method. This can be exposed via UDL with `constructor`, or via proc-macros with a `#[uniffi::constructor]` attribute.

Along with a default constructor, an interface can have named constructors, implemented as static functions.

Constructors:
* may omit or include the outer `Arc<>` - eg, we could have written `fn new() -> Arc<Self>`
* can return a `Result<>`
* can be async, although foreign language constraints means support for async primary constructors is patchy.

## Destructors

The foreign bindings will typically generate destructors, but regardless of the foreign semantics, they always hold an `Arc<>` to the Rust object, so these destructors will only drop their reference and may not drop the Rust object.

## Exposing Traits as interfaces

It's possible to have UniFFI expose a Rust trait as an interface.

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

Note UDL requires a [`Trait`](../udl/interfaces.md#traits) attribute; proc-macros `#[uniffi::export]` the trait declaration.

[Uniffi enforces all interfaces are `Send + Sync`](../internals/object_references.md#concurrency), meaning exported traits need to be explicitly bound.

References to traits are passed around like normal interface objects - in an `Arc<>`.
For example, your Rust would have these signatures:

```rust
fn get_buttons() -> Vec<Arc<dyn Button>> { ... }
fn press(button: Arc<dyn Button>) -> Arc<dyn Button> { ... }
```

### Foreign implementations

It's possible in both UDL (via a `[Foreign]` attribute) and proc-macros (via `#[uniffi::export(with_foreign)]`)
to declare the trait can also be implemented on the foreign side passed into Rust, for example:

```python
class PyButton(uniffi_module.Button):
    def name(self):
        return "PyButton"

uniffi_module.press(PyButton())
```

Note: This is currently only supported on Python, Kotlin, and Swift.

### Traits example

See the ["traits" example](https://github.com/mozilla/uniffi-rs/tree/main/examples/traits) for more.

## Exposing methods from standard Rust traits

Rust has a number of general purpose traits which add functionality to objects, such
as `Debug`, `Display`, etc. It's possible to tell UniFFI that your object implements these
traits and to generate FFI functions to expose them to consumers. Bindings may then optionally
generate special methods on the object.

For example, consider the following example:
```
[Traits=(Debug)]
interface TodoList {
    ...
};
```
and the following Rust code:
```rust
#[derive(Debug)]
struct TodoList {
   ...
}
```
(or using proc-macros)
```rust
#[derive(Debug, uniffi::Object)]
#[uniffi::export(Debug)]
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

    // Create a list of lists, one for each item this one
    sequence<TodoList> split();
};
```

To ensure that this is safe, UniFFI allocates every object instance on the heap using
[`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html), Rust's built-in smart pointer
type for managing shared references at runtime.

The use of `Arc` is transparent to the foreign-language code, but sometimes shows up
in the function signatures of the underlying Rust code.

When returning interface objects, UniFFI supports both Rust functions that wrap the value in an
`Arc<>` and ones that don't.  This only applies if the interface type is returned directly:

```rust
impl TodoList {
    // When the foreign function/method returns `TodoList`, the Rust code can return either `TodoList` or `Arc<TodoList>`.
    fn duplicate(&self) -> TodoList {
        TodoList {
            items: RwLock::new(self.items.read().unwrap().clone())
        }
    }

    // However, if TodoList is nested inside another type then `Arc<TodoList>` is required
    fn split(&self) -> Vec<Arc<TodoList>> {
        self.items.read()
            .iter()
            .map(|i| Arc::new(TodoList::from_item(i.clone()))
            .collect()
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
