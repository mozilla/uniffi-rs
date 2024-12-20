# Functions in UDL

All top-level *functions* get exposed through the UDL's `namespace` block.

The UDL file will look like:

```idl
namespace Example {
    string hello_world();
}
```

## Default values

Function arguments can be marked `optional` with a default value specified.

In the UDL file:

```idl
namespace Example {
    string hello_name(optional string name = "world");
}
```

## Async

Async functions can be exposed using the `[Async]` attribute:

```idl
namespace Example {
    [Async]
    string async_hello();
}
```
