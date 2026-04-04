`uniffi-bindgen-kotlin-jni` is an experimental bindgen system that generates
JNI-based Rust scaffolding and Kotlin bindings to call into it.

# Code generation

`uniffi-bindgen-kotlin-jni` uses `uniffi_parse_rs` to parse the Rust source
and create the metadata needed to generate the bindings/scaffolding.

The scaffolding is generated using the `uniffi_pipeline` framework and Askama templates
rather than from the macros.

Scaffolding is generated for a single crate only.
The main reason is that we only want to parse the source code once.

# FFI calling convention

## Primitive arguments / return values

If possible, we try to pass arguments directly using JNI.

The following types are passed as primitives:
  * Integers, floats, and bool.  Unsigned ints are converted to their signed counterparts.
  * `String` is passed as a `jstring`
  * `Vec`, `HashMap` and `HashSet` get passed as NIO byte buffer except for some special cases:
      * `Vec<T>` where T is a primitive type gets passed as an array.
  * Objects are passed as an `i64`
  * `Option<T>` where T is an integer type with 32-bit width or less
    We can use the niche optimization on these by casting them to an `i64`
    and using `i64::MAX` for `None`.
  * `Option<T>` where T is a floating point type.
    For these, we pick a special NaN value to use for `None`
  * `Option<String>` and `Option<Vec<u8>>` also uses the niche optimization
    with `null` used for `None`.
  * Flat enums are passed using their discriminants.

## Deconstructable types

If we can't pass types as primitives,
then we deconstruct the type into multiple FFI values and pass those.

* Records are deconstructed into their field's FFI values.
  Non-primitive fields are recursively deconstructed.
* Enums are passed as the union of the FFI values required for each variant,
  plus a `i32` value for the discriminant.
  If a variant doesn't have a FFI field, then a default value is passed.
  For example:
    * variant A contains `{a: i32, b: String, c: i64}`
    * variant B contains `{a: i32, b: i64}`
    * variant C contains `{a: i32, b: f32}`
    * The entire enum will be deconstructed to `(Int, String?, Long, Float)`
    * Variant B is passed as `(a, null, b, 0.0)`
* `Option<T>` that can't be passed as a primitive is passed as
  `bool` plus the FFI types for `T`.

Returning deconstructable types requires special handling that varies by the call type.
See notes on "Returning deconstructed types" below for details.

Note: if a type deconstructs to exactly 1 FFI type, then we simply return it as a primitive.

## FFI buffers

As described above, Vecs, HashMaps, and HashSets get passed as a nio ByteBuffer.
Here's how values are packed into those buffers.

* Primitive values are packed using native-endian.
  Container types ensure that primitive values are packed with
  the proper alignment.
* Pointers are packed as `u64` values to simplify the layout calculations.
* Records are packed field-by-field.
  The `FfiBufferLayout` class is used to track the current size of the buffer
  and the offsets needed to properly align fields.
* Enums variants are packed as a `i32` discriminant, followed by all variant fields.
  The packing algorithm is the same as for records.
  The size of the enum is the max size of all variants.
* Vecs, Hashmaps and HashSets are packed as nested buffers.
  The data/size fields are written as 2 `u64` values.
  `std::mem::forget` is used to avoid freeing the data until it's read.
  Both sides of the FFI must free these buffers immediately after reading from them.
* Strings are packed in a similar manner, except they're packed as 3 `u64` values
  (data/length/capacity).
* Box is also packed as a nested buffer.
  In this case we only need to pack a `u64` value for the pointer.

This strategy ensures that all types occupy a fixed size in the buffer,
which makes size calculations easy.
For example to allocate a vec, we calculate `item_size * length`.

As mentioned above Box values are packed as nested buffers.
This helps us implement the 2 main use-cases they're used for:
* Enable recursive types.
* Reduce wasted space when one enum variant is larger than the others.
  If you box the variant, then it will only contribute 8 bytes to the size of the enum.

All buffers are allocated by the Rust code, `ByteBuffer.allocateDirect` is never used.
This allows us to control when buffers get freed rather than rely on the garbage collector.

## Call details

### Kotlin -> Rust sync call

* Returning deconstructed types:
    * Deconstruct the value into primitives
    * Pass the primitives to the Kotlin lift function via JNI, getting back a `jobject`.
    * Return the resulting `jobject` back to Kotlin.
* Error handling: Rust constructs and throws an exception as described in `Errors/exceptions`

# Kotlin `uniffi` package

This is a generated package that contains all the FFI functions.
Putting these functions here has some advantages over our current system:

* **Less namespace pollution**.
  With `uniffi-bindgen-kotlin` have to add public functions for UniFFI-internal operations.
  These functions need to be public to make external types work.
  However, it feels slightly wrong for them to end up in the consumer-facing package.
  Putting these functions in the `uniffi` package feels better.
* **Less duplication**.
  If multiple crates use `Vec<u8>` in their interfaces, we'll only need to define 1 read and 1 write function.
* **Simplified external types**.
  We don't need to map namespaces to package names to find the right function.

# JNI

This crate uses the low-level `jni_sys` crate rather than the high-level `jni` crate.
This allows for more control and better performance in the JNI code.
An earlier version used `jni`, but benchmarks showed something like ~2-4x increase when switching to `jni_sys`.
`jni_sys` is not much harder for us to use, since we're operating at a very low level.

`jni_sys` is also a lighter-weight dependency,
which is important since we're forcing all consumer crates to depend on it.
