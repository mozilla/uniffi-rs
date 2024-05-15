# Describing the interface

There are two ways of describing your interface:
1 - with a UDL file (a type of IDL, Interface Definition Language);
2 - with [Rust procmacros](../proc_macro/index.md), similarly to how `wasm-bindgen` work. This avoids repeating your interface definitions in a separate file. Unfortunately, our docs aren't quite as good for that yet though.

## The UDL File

We describe in a UDL (a type of IDL, Interface Definition Language) file _what_ is exposed and available to foreign-language bindings. In this case, we are only playing with primitive types (`u32`) and not custom data structures but we still want to expose the `add` method.  
Let's create a `math.udl` file in the `math` crate's `src/` folder:

```idl
namespace math {
  u32 add(u32 a, u32 b);
};
```

Here you can note multiple things:

- The `namespace` directive: it will be the name of your Kotlin/Swift package. It **must** be present in any udl file, even if there aren't any exposed functions (e.g. `namespace foo {}`).
It will typically be your crate name.
- The `add` function is in the `namespace` block. That's because on the Rust side it is a top-level _function_, we will see later how to to handle _methods_.
- Rust's `u32` is also UDL's `u32`, but it is not always true! See the [Built-in Types](../udl/builtin_types.md) chapter for more information on mapping types between Rust and UDL.

**Note:** If any of the things you expose in the `udl` file do not have an equivalent in your Rust crate, you will get a hard error. Try changing the `u32` result type to `u64` and see what happens!
