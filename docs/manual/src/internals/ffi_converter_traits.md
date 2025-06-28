## Rust FFI conversion traits

UniFFI leverages a set of FFI converter traits to implement [lifting and lowering](./lifting_and_lowering.md) on the Rust side.
Each trait handles a single step in the lifting/lowering process (e.g. lifting an argument, lowering a return, etc.).
We implement these traits for each type used in the exported API then leverage them in the codegen.

For example, `uniffi::Lift` is used to lift values.
To handle a function like `fn print(msg: String)`, the generated code will use:

* `<String as Lift<crate::UniFfiTag>>::FfiType` when it needs to specify the FFI type (`RustBuffer` for strings).
* `<String as Lift<crate::UniFfiTag>>::try_lift()` when it needs to lift an argument value.  In this example, this means taking an FFI value (`<String as Lift<crate::UniFfiTag>>::FfiType` AKA `RustBuffer`) and converting it into a Rust `String` for passing to the Rust function.

Using a trait for this is important for proc-macros, which only see Rust tokens and don't know the surrounding context.
For example, if macros always used `RustBuffer` as the FFI type whenever it sees `String`, then that would fail if users created a type alias like `type MyTypeAlias = String`.
This may be unusual for `String`, but it's very common for `Result`.
In general, any reasoning about the tokens is fragile and should be avoided.

### `UniFfiTag` and the orphan rule

One odd part about the above code is that the `Lift` trait has a generic parameter which is always set to `crate::UniFfiTag`.
In general, all of the FFI converter traits have this parameter (i.e. we generate `Lower<crate::UniFfiTag>`, `LowerReturn<crate::UniFfiTag>`, etc.).
What's the point of all of this?

The main reason is to work around issues with the Rust orphan rule and types from 3rd-party crates.
For example, the [custom types](../types/custom_types.md) documentation shows how `url::Url` can be used in an exported API.
For these types, we normally can't implement `Lift` in the code we generate in the crate since neither `uniffi::Lift` or `url::Url` is local to that crate.
This same issue applies to all of the FFI converter traits.

To work around this we:

* Add a generic parameter to each trait (`Lift` becomes `Lift<UT>` where "UT" is short for `UniFfiTag`).
* Define a unit struct in each crate named `UniFfiTag` (the term "tag" is borrowed from the [C++ template pattern](https://www.geeksforgeeks.org/tag-dispatch-in-cpp/)).
* We use that unit struct as the generic parameter for the trait (e.g. `Lift<crate::UniFfiTag>` is used to lift a value).

Using the local type as a generic parameter means the impl no longer violates the orphan rule.
For details on this see the [Rust Chalk Book](https://rust-lang.github.io/chalk/book/clauses/coherence.html#the-orphan-rules-in-rustc)
The TLDR is that generic parameters "count" towards the requirement that there be a local type in the impl.

However, this makes it harder to use this impl from another crate.
UniFFI handles that in 2 ways:

* The `uniffi` crate generates blanket trait impls for all UniFFI tag params (`impl<UT> Lift<UT> for String`).
  This allows all crates to use them automatically with their `UniFfiTag` struct.
* UniFFI defines the `use_remote_type!` macro, which generates an implementation for the local
  `UniFfiTag` by forwarding to the implementation from another crate's `UniFfiTag`.
  See the [Remote and external types](../types/remote_ext_types.md#remote-external-types) for example usage.
  This is also what the `remote` flag of the [custom type macro](../types/custom_types.md) does.

### An incomplete list of FFI traits

UniFFI defines a large number of FFI conversion traits, each one used for a specific purpose.
This section describes a few them for explanatory purposes.
See `uniffi_core/src/ffi_converter_traits.rs` for a full and up-to-date list.

* `Lift`: Lift an value
* `Lower`: Lower a value
* `LowerReturn`: Lower a return value.
  * For most types this is equivalent `Lower`, but a specialized impl is created for `Result<T, E>`.
* `LiftRef`: Lift for a reference type.
  This is often just `Lift` then a borrow, but a specialized impl is created for `Arc<T>`.
* `FfiConverter`: General-purpose FFI conversion logic.
  When `FfiConverter` is defined on a type, all other FFI traits are automatically derived.
  This is what we implement for user-defined types like records and enums.
* `FfiConverterArc`: FfiConverter implementation for `Arc<T>`.
  This is another trait that we use to get around orphan rules.
  Crates can't directly implement `FfiConverter` on `Arc<T>` for some interface, so they implement `FfiConverterArc` instead.
  `uniffi` defines a blanket impl `FfiConverter` impl for these types (`impl<T: FfiConverterArc<UT>, UT> FfiConverter<UT> for Arc<T>`).
