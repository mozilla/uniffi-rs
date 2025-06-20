# Lifting, Lowering and Serialization

UniFFI is able to transfer rich data types back-and-forth between the Rust
code and the foreign-language code via a process we refer to as "lowering"
and "lifting".

Recall that UniFFI interoperates between different languages by defining
a C-style FFI layer which operates in terms of primitive data types and
plain functions. To transfer data from one side of this layer to the other,
the sending side "***lowers***" the data from a language-specific data type
into one of the primitive types supported by the FFI-layer functions, and the
receiving side "***lifts***" that primitive type into its own language-specific
data type.

Lifting and lowering simple types such as integers is done by directly casting the
value to and from an appropriate type. Non-trivial types such as Strings, Optionals and
Records, etc. are lowered to a byte buffer called a `RustBuffer` internally.

For example, a Python `str` is passed to Rust by lowering it to a `RustBuffer`, which is then
lifted to a Rust `String`

```mermaid
block-beta
  columns 2
  block:PythonLayer
    columns 1
    PythonTitle["Python"]
    PythonStr["str"]
  end
  block:RustLayer
    columns 1
    RustTitle["Rust"]
    RustString["String"]
  end
  block:FFILayer:2
    columns 3
    space:2
    FfiTitle["FFI"]
    space
    RustBuffer["RustBuffer"]
    space
  end

  PythonStr --"Lower"--> RustBuffer
  RustBuffer --"Lift"--> RustString

classDef default stroke:transparent,fill:#fff
classDef layer stroke:#c0c0c0,fill:#cae9ff
classDef title stroke:transparent,fill:transparent,color:#f72585
classDef invisible stroke:transparent,fill:transparent
class RustLayer layer
class PythonLayer layer
class FFILayer layer
class RustTitle title
class PythonTitle title
class FfiTitle title
```

## Lifting and Lowering when calling functions

As a concrete example, consider this interface for accumulating a list of integers:

```idl
namespace example {
  sequence<i32> add_to_list(i32 item);
}
```

Calling this function from foreign language code involves lowering the arguments, calling an FFI
function, lifting the arguments, then calling the original function.

```mermaid
block-beta
  columns 2
  block:PythonUserLayer
    columns 1
    PythonTitle["Generated Python function"]
    PythonFunc["add_to_list(int) -> [int]"]
  end
  block:RustUserLayer
    columns 1
    RustTitle["User-defined Rust Function"]
    RustFunc["add_to_list(i32) -> Vec<i32>"]
  end
  space:2
  block:ScaffoldingLayer:2
    columns 1
    ScaffoldingFunc["uniffi_fn_add_to_list(int32_t) -> RustBuffer"]
    ScaffoldingTitle["Generated Rust scaffolding function"]
  end

  PythonFunc -- "Lower" --> ScaffoldingFunc
  ScaffoldingFunc -- "Lift" --> RustFunc

classDef default stroke:transparent,fill:#fff
classDef layer stroke:#c0c0c0,fill:#cae9ff
classDef title stroke:transparent,fill:transparent,color:#f72585
classDef invisible stroke:transparent,fill:transparent

class PythonUserLayer layer
class RustUserLayer layer
class ScaffoldingLayer layer

class RustTitle title
class PythonTitle title
class ScaffoldingTitle title
```

Details:

1. UniFFI generates an `add_to_list` function in the foreign language (Python in the example
   diagram).  In this function the `item` argument and the return type are language-native types.
2. The generated function ***lowers*** each argument.  Since the `item` argument is a plain integer,
   it is lowered by casting to an `int32_t`.
3. The generated Python function passes the lowered arguments to the Rust scaffolding function.
   This is a `repr(C)` FFI function in Rust library and named `uniffi_fn_add_to_list` in this example.
4. The Rust scaffolding function ***lifts*** each argument received over the FFI into a native Rust type.
   Since `item` is a plain integer no conversion is needed.
5. The Rust scaffolding passes the lifted arguments to the user-defined Rust `add_to_list` function, which then executes normally.
6. The Rust scaffolding function receives the return value and now needs to ***lower*** the it to pass it back across the FFI.
   Since this type's `::FfiType` is a `RustBuffer`, it's lowered by serializing the values into a byte buffer (`RustBuffer`), which is then returned.
7. The generated Python function receives the return value, and then ***lifts*** it to a native data type.
   Since this type's `::FfiType` is a `RustBuffer`, it's lifted by deserializing a language-native list of integers from the RustBuffer.

## Lowered Types

| UDL Type | Representation in the C FFI |
|----------|-----------------------------|
| `i8`/`i16`/`i32`/`i64` | `int8_t`/`int16_t`/`int32_t`/`int64_t` |
| `u8`/`u16`/`u32`/`u64` | `uint8_t`/`uint16_t`/`uint32_t`/`uint64_t` |
| `f32`/`float` | `float` |
| `f64`/`double` | `double` |
| `boolean` | `int8_t`, either `0` or `1` |
| `string` | `RustBuffer` struct pointing to utf8 bytes |
| `bytes` | Same as `sequence<u8>` |
| `timestamp` | `RustBuffer` struct pointing to a i64 representing seconds and a u32 representing nanoseconds |
| `duration` | `RustBuffer` struct pointing to a u64 representing seconds and a u32 representing nanoseconds |
| `T?` | `RustBuffer` struct pointing to serialized bytes |
| `sequence<T>` | `RustBuffer` struct pointing to serialized bytes |
| `record<string, T>` | `RustBuffer` struct pointing to serialized bytes |
| `enum` and `[Enum] interface` | `RustBuffer` struct pointing to serialized bytes |
| `dictionary` | `RustBuffer` struct pointing to serialized bytes |
| `interface` / `callback interface ` / `trait interface` | `u64` See [object references](./object_references.md) |


## Serialization Format

When serializing complex data types into a byte buffer, UniFFI uses an
ad-hoc fixed-width format which is designed mainly for simplicity.
The details of this format are internal only and may change between versions of UniFFI.

| UDL Type | Representation in serialized bytes |
|----------|-----------------------------|
| `i8`/`i16`/`i32`/`i64` | Fixed-width 1/2/4/8-byte signed integer, big-endian|
| `u8`/`u16`/`u32`/`u64` | Fixed-width 1/2/4/8-byte unsigned integer, big-endian |
| `f32`/`float` | Fixed-width 4-byte float, big-endian |
| `f64`/`double` | Fixed-width 8-byte double, big-endian |
| `boolean` | Fixed-width 1-byte signed integer, either `0` or `1` |
| `string` | Serialized `i32` length followed by utf-8 string bytes; no trailing null |
| `T?` | If null, serialized `boolean` false; if non-null, serialized `boolean` true followed by serialized `T` |
| `sequence<T>` | Serialized `i32` item count followed by serialized items; each item is a serialized `T` |
| `record<string, T>` | Serialized `i32` item count followed by serialized items; each item is a serialized `string` followed by a serialized `T` |
| `enum` and `[Enum] interface` | Serialized `i32` indicating variant, numbered in declaration order starting from 1, followed by the serialized values of the variant's fields in declaration order |
| `dictionary` | The serialized value of each field, in declaration order |
| `interface` | Fixed-width 8-byte unsigned integer encoding a pointer to the object on the heap |

Note that length fields in this format are serialized as *signed* integers
despite the fact that they will always be non-negative. This is to help
ease compatibility with JVM-based languages since the JVM uses signed 32-bit
integers for its size fields internally.
