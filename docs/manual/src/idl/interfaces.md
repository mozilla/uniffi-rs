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
