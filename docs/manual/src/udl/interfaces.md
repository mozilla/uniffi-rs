# Interfaces in UDL

You expose our [`TodoList` example](../types/interfaces.md) in UDL like:

```idl
interface TodoList {
    constructor();
    void add_item(string todo);
    sequence<string> get_items();
};
```

The `constructor()` calls the Rust's `new()` method.

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
    // This alternate constructor is async.
    [Async, Name=new_async]
    constructor(sequence<string> items);
    ...
};
```

For each alternate constructor, UniFFI will expose an appropriate static-method, class-method or similar
in the foreign language binding, and will connect it to the Rust method of the same name on the underlying
Rust struct.

Constructors can be async, although support for async primary constructors in bindings is minimal.

## Traits

It's possible to have UniFFI expose a Rust trait as an interface by specifying a `Trait` attribute.

For example, in the UDL file you might specify:

```idl
[Trait]
interface Button {
    string name();
};

namespace traits {
    sequence<Button> get_buttons();
    Button press(Button button);
};
```

### Foreign implementations

Use the `WithForeign` attribute to allow traits to also be implemented on the foreign side passed into Rust, for example:

```idl
[Trait, WithForeign]
interface Button {
    string name();
};
```

would allow foreign implementations of that trait - eg, passing one back into Rust from Python:

```python
class PyButton(uniffi_module.Button):
    def name(self):
        return "PyButton"

uniffi_module.press(PyButton())
```

### Traits construction

Because any number of `struct`s may implement a trait, they don't have constructors.
