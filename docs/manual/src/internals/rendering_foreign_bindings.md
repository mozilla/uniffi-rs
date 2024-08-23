# Rendering Foreign Bindings

This document details the general system that UniFFI uses to render the foreign bindings code.

A language binding has to generate code for two separate but entangled requirements:

* Generate the API in the target language.
* Implement the FFI - every type needs an FfiConverter.

## API generation

Our foreign bindings generation uses the [Rinja](https://rinja.readthedocs.io/en/stable/) template rendering engine. Rinja uses
a compile-time macro system that allows the template code to use Rust types directly, calling their methods passing them
to normal Rust functions.

The templates have access to `ci`, a [`ComponentInterface`](https://github.com/mozilla/uniffi-rs/blob/main/uniffi_bindgen/src/interface/mod.rs), which is the Rust representation of all the UniFFI types in your crate.

The task of the templates is to render `ci` into a "module" for the foreign binding.  This mainly consists of rendering support for each [`Type`](https://github.com/mozilla/uniffi-rs/blob/main/uniffi_meta/src/types.rs) described in your crate.

Eg, here's where [Python uses `ci` to iterate over the types](https://github.com/mozilla/uniffi-rs/blob/884f7865f3367c494e9165e21c1255018577db01/uniffi_bindgen/src/bindings/python/templates/Types.py#L3)

The templates create foreign-native types for everything from ffi-native types (int/etc) to functions, dictionaries etc. The implementation of these generated types might call your your Rust implemented FFI, as described below.

Bidings also need to do alot of work to make language identifiers etc work correctly - eg, turn `this_func(this_arg: ThisType)` into `thisFunc(...)`

## Breaking down a Rust function called by Python.

Let's take a look at where [Python generates a top-level public function](https://github.com/mozilla/uniffi-rs/blob/884f7865f3367c494e9165e21c1255018577db01/uniffi_bindgen/src/bindings/python/templates/TopLevelFunctionTemplate.py#L37-L40+).

This will generate code like the following:

```
def this_func(this_arg=0) -> None:
```

Let's break the template down:

```
def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}) -> None:
```

The Rinja language uses double-curly braces (`{ }`) to interpolate blocks of code into the string output.

`{{ func.name()|fn_name }}` becomes `this_func`: [It calls the `name` method on a `Function` object](https://github.com/mozilla/uniffi-rs/blob/884f7865f3367c494e9165e21c1255018577db01/uniffi_bindgen/src/interface/function.rs#L72) (you can see all the other metadata about functions there too).
Rinja uses a "filter" concept: Functions that take the value left of the pipe operator (`|`) to produce a new value.
The "filter" used in the above template is called`fn_name` and is [defined in the Python bindings generator](https://github.com/mozilla/uniffi-rs/blob/884f7865f3367c494e9165e21c1255018577db01/uniffi_bindgen/src/bindings/python/gen_python/mod.rs#L567) - which ends up just handing the fact it might be a Python keyword but otherwise returns the same value.

`{%- call py::arg_list_decl(func) -%}`: Calling an Rinja macro, passing the `func` object linked above. It knows how to turn the function arguments into valid Python code.

Skipping a few lines ahead in that template, we call the FFI function `{% call py::to_ffi_call(func) %}` - which ultimately
end up a call to an `extern "C"` FFI function you generated named something like `uniffi_some_name_fn_func_this_func(...)`

The bindings also need to do lots of administrivia - eg, calling initialization functions, importing dependencies, etc

### Implementing the FFI.

[All types](https://github.com/mozilla/uniffi-rs/blob/884f7865f3367c494e9165e21c1255018577db01/uniffi_meta/src/types.rs#L62) must implement an FFI converter.

The FfiConverter is described in the [Lifting, Lowering and Serialization](./lifting_and_lowering.md) chapter.
Note that this means different things for "native" types (`int`, etc), but otherwise there's a lot of `RustBuffer`!
eg, [the Swift `Bool`](https://github.com/mozilla/uniffi-rs/blob/884f7865f3367c494e9165e21c1255018577db01/uniffi_bindgen/src/bindings/swift/templates/BooleanHelper.swift#L1C39-L1C51) vs [Swift record/struct support](https://github.com/mozilla/uniffi-rs/blob/884f7865f3367c494e9165e21c1255018577db01/uniffi_bindgen/src/bindings/swift/templates/RecordTemplate.swift#L38)

## FFI Functions

Above, we mentioned your template will generate a call to, eg, `uniffi_some_name_fn_func_this_func`.
This function is automatically generated and made public in your Rust crate - it's a function that might look like:

```
pub extern "C" fn uniffi_some_name_fn_func_this_func(
    arg: i32,
    call_status: &mut ::uniffi::RustCallStatus,
) -> i32 {
```

The bindings need to use the metadata to create the correct args to make these calls using the FFI converter implementations.

There will be a number of memory/lifetime/etc "adminstrative" FFI functions that will also be used by the generated implementation.
