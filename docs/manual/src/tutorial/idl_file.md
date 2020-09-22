# The IDL file

We describe in a IDL (Interface Definition Language) file *what* is exposed and available to foreign-language bindings. In this case, we are only playing with primitive types (`u32`) and not custom data structures but we still want to expose the `add` method.  
Let's create a `math.idl` file in the `math` crate's `src/` folder:

```idl
namespace math {
  u32 add(u32 a, u32 b);
};
```

Here you can note multiple things:
- The `namespace` directive: it will be the name of your Kotlin/Swift package. It **must** be present in any idl file, even if there ain't any exposed function (e.g. `namespace foo {}`).
- The `add` function is in the `namespace` block. That's because on the Rust side it is a top-level *function*, we will see later how to to handle *methods*.
- Rust's `u32` is also IDL's `u32`, but it is not always true! (TODO table correspondance)

**Note:** If any of the things you expose in the `idl` file do not have an equivalent in your Rust crate, you will get a hard error. Try changing the `u32` result type to `u64` and see what happens!
