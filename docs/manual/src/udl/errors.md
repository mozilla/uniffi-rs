# Errors in UDL

You expose our [error example](../types/errors.md) in UDL like:

```
[Error]
enum ArithmeticError {
  "IntegerOverflow",
};


namespace arithmetic {
  [Throws=ArithmeticError]
  u64 add(u64 a, u64 b);
}
```

Note that in the above example, `ArithmeticError` is "flat" - the associated
data is not exposed - the foreign bindings see this as a simple enum-like object with no data.
If you want to expose the associated data as fields on the exception, use this syntax:

```
[Error]
interface ArithmeticError {
  IntegerOverflow(u64 a, u64 b);
};
```

# Interfaces as errors

In our [error interface example](../types/errors.md) we are throwing the object `MyError`.

```idl
namespace error {
  [Throws=MyError]
  void bail(string message);
}

[Traits=(Debug)]
interface MyError {
  string message();
};
```
