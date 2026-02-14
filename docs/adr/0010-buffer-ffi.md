# Buffer FFI

* Status: proposed
* Deciders:
* Date: 2024-05-24

Discussion and approval: [PR 2816](https://github.com/mozilla/uniffi-rs/pull/2816)

## Context and Problem Statement

Our current FFI is based on passing arguments/return values using the C ABI.
This forces languages like Kotlin, Python, and JS that can't make C calls directly
to use an intermediate layer to make these calls.
Usually this means a libffi-based library, like ctypes or JNA.
However, there are several issues with this approach:

* Performance can be poor especially when using JNA.
* JNA is has been a persistent source of issues on Kotlin.
  There are a couple current issues without clear solutions: #2740, #2624.
* Limitations in these libraries limit how we can design the FFI.
  For example, callback methods can't return values directly because of https://bugs.python.org/issue5710.

We want to design a new FFI that avoids these issues as much as possible.

## Scope

This document discussion a new FFI for languages like Kotlin, Python, and JS that can't make C calls directly.
This hopefully helps us move toward a stable "1.0" FFI,
but this document is not proposing that we freeze the API at this point.

Languages like Swift that can make C calls directly are not considered.
Maybe we will continue to use the current FFI for those languages or maybe we'll develop another new FFI.

The exact mechanism for making these calls is also out-of-scope for this ADR.
For example, Kotlin and Java may use JNA or JNI.

## Decision Drivers

## Decisions

This document is organized around several independent decisions rather than a single one.
Hopefully this makes each decision clearer.

### Using a buffer to pass FFI values

The main proposal is to change the general form for FFI calls to:

```
extern "C" uniffi_buffer_ffi_function_name(ffi_buffer: *u8) {
    // ... code here
}
```

Each FFI call inputs a single buffer that's used for both input arguments and return values.
Let's name the FFI the "Buffer FFI" and the function argument the "FFI buffer".
To avoid conflicts with the current FFI, all FFI functions should be prefix with `uniffi_buffer_`

Callees should:

 * Read all arguments from the buffer
 * Lift the argument data into high-level types
 * Call the exported function using the lifted arguments
 * Lower the return value into an FFI type
 * Write the result the buffer:
    * For successful calls, write `0` followed by the return value.
    * For expected errors, write `1` followed by the error value
    * For unexpected errors, write `2` followed by a `RustBuffer` containing a error message.

#### Packing primitive values to the FFI buffer

* Ints and floats are packed in native-endian format.
* Pointers are casted to `u64` values then packed into the buffer.
  Function pointers are handled the same way.
* All items are aligned to 64-bit addresses.

#### Allocating the buffer

Callers must ensure the buffer is large enough to hold all arguments as well as the return value
(one or the other, not both at once).
Callers may use different strategies to allocate/free the FFI buffer, including:

* Allocating a new buffer for each call
* Creating an array on the stack
* Statically allocating a single buffer for each thread in a thread-local variable

In order to support all of these strategies, callers must read all data from the buffer immediately
before there's any chance of another call across the FFI.

As long as no dynamically sized values are present in the argument list or return value,
then the buffer will have a fixed and relatively small size.
Buffers for dynamically sized values are discussed in the "RustBuffer and heap data" section below.

#### Performance

The performance of the buffer FFI has been well tested on Kotlin 
with benchmarks showing that it performs much faster than the current FFI.
See the appendix below for details,
the TLDR is that it speeds up most calls by a factor of 100x or so.

### Using JNI/pyo3 to pass FFI values

The main alternative to the buffer FFI that was considered
was using a language-specific bindings layer like JNI or pyo3.
Defining a Python module using the C API is another potential path.
We mostly focused on JNI and this section will reflect that.
However, it's expected that the same logic applies to pyo3 and other systems.

This would mean generating Rust code that looked like this:

```
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_some_package_name_UniffiLibrary_rustFunc(
    mut env: JNIEnv,
    _class: JClass,
    int_arg: i32,
    vec_of_ints: JObject,
    struct_arg: JObject,
    vec_of_structs: JObject,
) -> f64 {
    // Note: no lifting needed for `int_arg`

    // Lift the vec_of_ints arg
    let vec_of_ints_elements = env.get_array_elements_critical(vec_of_ints, ReleaseMode::NoCopyBack).unwrap();
    let vec_of_ints_lifted = vec_elements.iter().collect();

    // Lift the struct arg
    let field_ids = STRUCT_FIELD_IDS.get().unwrap();
    let struct_lifted = TheStruct {
        field_a: env.get_field_unchecked(
            &struct_arg, 
            field_ids.0,
            ReturnType::Primitive(Primitive::Int),
        ).unwrap().i().unwrap(),
        field_b: env.get_field_unchecked(
            &struct_arg, 
            field_ids.1,
            ReturnType::Primitive(Primitive::Double),
        ).unwrap().d().unwrap(),
    };

    // Lift the vec_of_structs arg
    // Unfortunately, we need to get each element one at a time rather in this case.
    let vec_of_structs_length = env.get_array_length(vec_of_structs).unwrap();
    let vec_of_structs_lifted = (0..vec_of_structs_length).iter()
        .map(|i| {
            let struct = env.get_object_array_element(vec_of_structs, i);
            TheStruct {
                field_a: env.get_field_unchecked(
                    &struct, 
                    field_ids.0,
                    ReturnType::Primitive(Primitive::Int),
                ).unwrap().i().unwrap(),
                field_b: env.get_field_unchecked(
                    &struct, 
                    field_ids.1,
                    ReturnType::Primitive(Primitive::Double),
                ).unwrap().d().unwrap(),
            }
        })
        .collect::<Vec<_>>();

    // Call the Rust function
    let result = rust_func(int_arg, vec_of_ints_lifted, struct_lifted, vec_of_structs_lifted);

    // ...lower and return the return value, handle errors, etc.
}
```

The corresponding generated Kotlin code would be fairly simple,
since a lot of the lowering is happening in the JNI layer.

JNI code performs better than buffers in some cases:

* **Primitive values** (ints, floats, bools, etc).  Benchmarks show about a 20% speedup.
* **Arrays of primitives**.
  This has not been tested, but it seems safe to assume the JNI approach is faster.

However, buffers perform faster in other cases:

* **Structs**.
  It's slightly faster to read elements from a buffer than to make a JNI function call for each field.
* **Enums**.
  In addition to needing JNI cals per field, but you also need JNI calls to figure out the enum variant.
  Calling `IsInstanceOf` for each variant seems very slow (although this has not been tested).
* **Nested data** (vecs of structs, hash maps, structs with enum fields, etc).
  At this point buffers start to significantly out-perform JNI
  since you need to make more than 1 JNI call per item.
  For example, with a vec of structs you need to make an extra JNI call per item in addition to the
  JNI calls for each field.

See https://github.com/mozilla/uniffi-rs/issues/2672 for further discussion and the rough
benchmarks.

JNI supports passing nio buffers to the C functions which we could use to speed up these cases.
We could use the nio buffers for cases from the second list
and "normal JNI" for cases in the first list.

However, if the only cases where you don't want a buffer are primitives and arrays of primitives,
you start to wonder why not use a buffer for everything?
The performance difference will often be negligible
since primitive values are already quite fast relative to structs/enums,
and the code generation would be simplified.
This would essentially mean we're back to the buffer FFI approach, but using JNI to implement it.

### Options

* [A1] Buffer FFI
   * Good, because it creates simple FFI signatures which will help to avoid JNA bugs
     and workaround limitations like Python ctypes prohibiting callbacks from returning structs.
   * Good, because it has nearly the best performance.
   * Good, because we get more control around the FFI.
     For example, we've historically always had to pass structs and enums using a `RustBuffer`
     because it was deemed to complex to pass them over the C-ABI.
     However, it's not hard to pack structs/enums into the FFI buffer -- see below.
   * Good, because we can eliminate the `RustCallStatus` out pointer.
     This only exists because we can't handle enums in the current FFI.
   * Good, because all FFI functions have the same signature, which can simplify the codegen.
     In particular, I think it could significantly improve `uniffi-bindgen-gecko-js`
     which needs to generate a C++ layer to allow JS to call Rust scaffolding functions.
     Maybe we could replace some or all of that layer with 2 functions:
     `get_scaffolding_function(name: String) -> ScaffoldingFunction` and
     and `call_scaffolding_function(func: ScaffoldingFunction, buf: ArrayBuffer)`.
     These could call any UniFFI scaffolding function regardless of the signature of the underlying Rust function.
   * Bad, because we need to cast/serialize data pointers and function pointers to the buffer.
     This makes pointer providence trickier and could cause issues on exotic platforms
     where the pointer width is greater than 64 bits.

* [A2] Current FFI
   * Good, because we've already implemented it.
 
* [A3] Language-specific FFIs, like JNA/pyo3
   * Bad, because it performs poorly for structs/enums/nested data.
   * Bad, because each language needs specialized generated Rust code.
   
* [A4] A3, but using a buffer for structs/enums
   * Good, because has the best performance.
   * Bad, because it's more complex than A1.
   * Bad, because each language needs specialized generated Rust code.

#### Decision Outcome

Option [A1] Buffer FFI

#### Decision Drivers

* Simple codegen is more important than the absolute fastest performance
* The JNA-related crashes are bad enough that we should move away from the current FFI.

### Passing Structs and Enums

We currently pass structs and enums using a `RustBuffer`.
This was to avoid the complexity of defining JNA/ctypes subclasses for each time.
Enums provide an extra source of complexity, since they require definning a tagged union.

However, with the buffer FFI, we can easily pass struct/enums directly.
For structs, we simply serialize each field in order.
For enums, we serialize the tag as a `u64` value, then pack each field of the variant.
This process can be applied recursively for nested structs/enums.

Another option would be to pass structs and enums using a reference.
This could improve performance for large structs/enums by avoiding a copy.
However, this is only possible if we know the field layout.

* [B1] Pass structs/enums using RustBuffers
* [B2] Serialize structs/enums fields into the FFI buffer
  * Good, because it avoids a RustBuffer allocation for these values
  * Bad, because it can lead to extra RustBuffer allocations for child values.
    However, see the next section for how we can avoid this.
* [B3] Pass structs/enums using references
  * Good, because it can avoid a copy in some cases
  * Bad, because only works when we know the field layout.
    In practice this means it only works for Rust -> FFI calls.
    Furthermore, we'd have to handle `#[repr]` attributes somehow.
  * Bad, because it adds significant complexity to the FFI.
  
#### Decision Outcome

Option [B2] Serialize structs/enums fields into the FFI buffer

### RustBuffer and heap data

One issue with our current FFI is that we can allocate multiple RustBuffers per call
and this issue could get even worse with the proposed changes.

We currently allocate a RustBuffer for values that require heap allocation (vecs, hash-maps, strings),
as well any any structs/enums values (including `Option`).
After the change, we don't need to allocate a RustBuffer for structs/enums
but that ironically may increase the total number of allocations.
If a struct or enum has N fields that are passed using a RustBuffer,
then could mean allocating N RustBuffers where before we only allocated 1.

To avoid all of this, use a single buffer for the arguments if any is a heap value.
This will be used instead of the normal FFI buffer for all arguments.
Caller will allocate this buffer on the heap before the call and free it afterwards.

#### Returning heap values

If the return value is a heap value, the callee will allocate a new RustBuffer to store it.
The callee will write its fields to the FFI buffer like any return.
The caller is responsible for freeing the return buffer once the return value has been lifted.

#### Packing vecs/hash-maps/strings to the FFI buffer

For each of these, first pack the length of the value as a `u64`.
Then pack each vec item, string byte, or map key/value pair in order.
All items will be aligned to 64-bit boundaries, except string bytes.
Use native-endian when packing items.

#### Optimizing particular cases

There are several optimizations could avoid RustBuffer allocations in some cases.
For example, passing strings as a pointer/length pair or passing small objects using the normal FFI buffer.
We may optimize for these cases in the future, but this ADR doesn't make any proposals.

### Options

* [C1] One RustBuffer per heap type
* [C2] Single buffer for all arguments
  * Good, because it limits the number of allocations
  * Good, because it simplifies the memory management
    Callers only need to free a single buffer rather than multiple ones
* [C3] Avoid allocations for certain cases
  * Good, because it can eliminate allocations altogether
  * Good, because it can eliminate copies
  * Bad, because it adds extra complexity to the FFI
  
#### Decision Outcome

Option [C2] Single buffer for all arguments

### Low-hanging fruit

Finally, there are obvious improvements that we can make for the new FFI.
These all feel obvious, so they're simple listed here without much discussion:

* For functions that can't fail, don't pass in a `RustCallStatus` argument
* Make `RustBuffer` functions use primitive values, rather than a `RustBuffer` struct:
  * `rustbuffer_alloc` can return a pointer since the caller knows the length/capacity
  * `rustbuffer_free` can input the pointer/length/capacity as separate arguments, rather than the struct.

## Appendix

### Performance testing

When designing this FFI, we did a lot of Kotlin performance testing to compare the different possibilities.
Kotlin was chosen because it has the worst performance of all builtin bindings.
The buffer FFI should also improve performance for Python and other languages,
but the amount will be less than for Kotlin.
Here's a summary of that testing:

* The first step was to generate the Kotlin/Rust code and check that in.
  Other commits made changes to the generated code.
  This made experimentation faster and made it easier to see how changes affected the FFI.
* https://github.com/bendk/uniffi-rs/commit/push-knutvwvsuxxn
* The "low-hanging fruit" changes decreased benchmark times by about 50% in most cases
  (https://github.com/bendk/uniffi-rs/commit/push-xmprvxqrzxpv)
  * Another potential low-hanging fruit change would be to allocate buffers in Kotlin when we can
    instead of using the RustBuffer FFI.
    However, this didn't show any real performance benefits.
    (https://github.com/bendk/uniffi-rs/commit/push-uvylovvzslrl).
* Switching to an buffer FFI further improved performance.
  Many of the times decreased by about 98%, though some decreased less.
  The `nested-data` benchmark regressed because of the issue mentioned above
  where the new FFI caused more RustBuffer allocations. 
  (https://github.com/bendk/uniffi-rs/commit/push-ouylqyrnpnqr).
  * Using JNI to pass JVM values directly was also tested as an alternative.
    Benchmarks showed similar performance to the buffer FFI with JNA,
    but worse performance compared to the buffer FFI with JNI.
    (https://github.com/bendk/uniffi-rs/commit/push-nkkpuuuvonow and
    https://github.com/bendk/uniffi-rs/commit/push-qplztoxoqwny).
  * Testing shows that using `sun.jna.Pointer` was much faster than `java.nio.ByteBuffer`
    for reading/writing to the buffer.
    (https://github.com/bendk/uniffi-rs/commit/push-tvmtokymtoyp).
* Using a single RustBuffer for heap allocations improved performance
  for benchmarks that passed vecs/maps/strings.
  The `strings` benchmark time decreased by about 45%.
  The `nested-data` benchmark decreased by 80% which is an overall speedup compared to the
  low-hanging fruit commit.
  (https://github.com/bendk/uniffi-rs/commit/push-mlzzlsxznylp)
* When compared to the current code, all benchmark times improved
  and criterion usually reported speedups around -99%, meaning the code ran ~100x faster.
