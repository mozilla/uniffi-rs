# Interfaces/Objects

Interfaces are represented in the Rust world as a struct with an `impl` block containing methods. In the Kotlin or Swift world, it's a class.  
Because Objects are passed by reference and Dictionaries by value, in the uniffi world it is impossible to be both an Object and a [Dictionary](./structs.md).

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

`uniffi` will generate these proxies of live objects with an interface or protocol.

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

## Concurrent Access

Since interfaces represent mutable data, uniffi has to take extra care
to uphold Rust's safety guarantees around shared and mutable references.
The foreign-language code may attempt to operate on an interface instance
from multiple threads, and it's important that this not violate Rust's
assumption that there is at most a single mutable reference to a struct
at any point in time.

Uniffi enforces this using runtime locking. Each interface instance
has an associated lock which is transparently acquired at the beginning of each
call to a method of that instance, and released once the method returns. This
approach is simple and safe, but it means that all method calls on an instance
are run in a strictly sequential fashion, limiting concurrency.

You can read more about the technical details in the docs on the
[internal details of managing object references](../internals/object_references.md).