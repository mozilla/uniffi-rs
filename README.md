# uniffi

A very hand-wavy idea about autogenerating our FFI code and its bindings.
Don't get your hopes up just yet ;-)

## What?

Our current approach to building shared components in rust involves writing a lot
of boilerplate code by hand, manually flattening the rust code into an `extern "C"`
FFI layer and then manually importing and exposing that into each of Kotlin, Swift
and XPCOM+JS. The process is time-consuming, error-prone, and boring.

What if we didn't have to write all of that by hand?

In an aspirational world, we could get this kind of easy cross-language interop for
free using [wasm_bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) and
[webassembly interface types](https://hacks.mozilla.org/2019/08/webassembly-interface-types/) -
imagine writing an API in Rust, annotating it with some `#[wasm_bindgen]` macros,
compiling it into a webassembly bundle, and being able to import and use that bundle
from any target language!

That kind of tooling is not available to shipping applications today, but that doesn't
mean we can't take a small step in that general direction.

### Key Ideas

* Specify the component API using an abstract *Interface Definition Language*.
* When implementing the component:
  * Process the IDL into some Rust code scaffolding to define the FFI, data classes, etc.
  * Have the component crate `!include()` the scaffolding and fill in the implementation.
* When using the component:
  * Process the IDL to produce FFI bindings in your language of choice
  * Use some runtime helpers to hook it up to the compiled FFI from the component crate.


## Component Interface Definition

We'll abstractly specify the API of a component using the syntax of [WebIDL](https://en.wikipedia.org/wiki/Web_IDL),
but without getting too caught up in matching its precise semantics. This choice is largely driven by
the availability of quality tooling such as the [weedle crate](https://docs.rs/weedle/0.11.0/weedle/),
general familiarity around Mozilla, and the desire to avoid bikeshedding any new syntax.

We'll model the *semantics* of the component API loosely on the primitives defined by the
[Wasm Interface Types](https://github.com/WebAssembly/interface-types/blob/master/proposals/interface-types/Explainer.md)
proposal (henceforth "WIT"). WIT aims to solve a very similarly-shaped problem to the one we're faced
with here, and by organizing this work around similar concepts, we might make it easier to one day
replace all of this with direct use of WIT tooling.

In the future, we may be able to generate the Interface Definition from annotations on the rust code (in the style of `#[wasm_bindgen]`) rather than from a separate IDL file. But it's much easier to get
started using a separate file.

The prototype implementation of this is in [./src/types.rs](./src/types.rs).
See [fxa-client.idl](../../fxa-client/fxa-client.idl) for an example of an interface definition.

### Primitive Types

We'd avoid WedIDL's sparse and JS-specific types and aim to provide similar primitive types
to the WIT proposal: strings, bools, integers of various sizes and signedeness. We already know
how to pass these around through the FFI and the details don't seem very remarkable.

They all pass by value (including strings, which get copied when transiting the FFI).

### Object Types (a.k.a. Reference Types, Handle Types, Structs, Classes, etc)

These represent objects that you can instantiate, that have opaque internal state and methods that
operate on that state. They're typically the "interesting" part of a component's API. We currently
implement these by defining a Rust struct, putting instances of it in a `ConcurrentHandleMap`, and
defining a bunch of `extern "C"` functions that can be used to call methods on it.

In WebIDL these would be an `interface`, like so:

```
interface MyObject {
  constructor(string foo, bool isBar);
  bool checkIfBar();
}
```

I don't think the WIT proposal has an equivalent to these types; they're kind of like an
`anyref` I guess? We should investigate further...

In the FFI, these are represented by an opaque `u64` handle.

When generating component scaffolding, we could transparently create the HandleMap and the `extern "C"` functions that operate on it. We'd rely on the component code to provide a corresponding `MyObject` struct, and on Rust's typechecker to complain if the implemented methods on that struct don't match the expectations of the generated scaffolding.

When generating language-specific bindings, this becomes a `class` or equivalent.

TODO:
* Can we use member attributes to annotate which methods require mutable vs shared access?
* Can we use member attributes to identify which methods may block, and hence should be turned into a deferred/promise/whatever.

### Record Types (a.k.a. Value Types, Data Classes, Protobuf Messages, etc)

These are structural types that are passed around *by value* and are typically only used
for their data. The sorts of things that we currently use JSON or Protobuf for in the FFI.

In WebIDL this corresponds to the notion of a `dictionary`, which IMHO is not a great
name for them in the context of our work here, but will do the job:

```
dictionary MyData {
  required string foo;
  u64 value = 0;
}
```

In the WIT proposal these are "records" and we use the same name here.

When generating the component scaffolding, we'd do a similar job to what's done with protobuf
today - turn the record description into a rust `struct` with appropriate fields, and helper
methods for serializing/deserializing, accessing data etc.

When generating language-specific bindings, records become a "data class" or similar construct,
with field access and serialization helpers. Again, much like we currently do with protobufs.

When passing back and forth over the FFI, records are serialized to a byte buffer and
deserialized on the other side. We could continue to use protobuf for this, but I suspect
we could come up with a simpler scheme given we control both sides of the pipe. Callers
passing a record must keep the buffer alive until the callee returns; callers receiving
a record must call a destructor to free the buffer after hydrating an object on their side.

### Sequences

Both WebIDL and WIT have a builtin `sequence` type and we should use it verbatim.

```
interface MyObject {
    sequence<Foo> getAllTheFoos();
}
```

We currently use ad-hoc Protobuf messages for this, e.g. the `AccountEvent` and
`AccountEvents` types in fxa-client. But it seems reasonable to support a generic
implementation on both sides of the FFI boundary.

When traversing the FFI, these would be serialized into a byte buffer and parsed
back out into a Vec or Array or whatever on the other side. Just like Record types.

### Enums

WebIDL as simple C-style enums, like this:

```
enum AccountEventType {
  "INCOMING_DEVICE_COMMAND",
  "PROFILE_UPDATED",
  "DEVICE_CONNECTED",
  "ACCOUNT_AUTH_STATE_CHANGED",
  "DEVICE_DISCONNECTED",
  "ACCOUNT_DESTROYED",
};
```

These could be encoded into an unsigned integer type for transmission over the FFI,
like the way we currently handle error numbers.

There is also more sophisticated stuff in there, like union types and nullable
types. I haven't really thought about how to map those on to what we need.

### Callbacks

WebIDL has some syntax for them, but I haven't looked at this in any detail
at all. It seems hard, but also extremely valuable because handling callbacks
across the FFI boundary has been a pain point for us in the past.

## Scaffolding Generation

Currently a very hacky attempt in [./src/scaffolding.rs](./src/scaffolding.rs),
and a `generate_component_scaffolding(idl_file: &str)` function that's intended
to be used from the component's build file.

See the [fxa-client crate](../../fxa-client/build.rs) for an example of (attempting to)
use this, although it's in a very incomplete state.

It doesn't produce working Rust code, but it'll produce Rust-shaped kind-of-code
that gives you a bit of an idea what it's going for.

Could really benefit from a templating library rather than doing a bunch of
`writeln!()` with raw source code fragements.

## Language Bindings Generation

Currently totally unimplemented.

A great opportunity for anyone interested to
dive in! You could try looking at the hand-written Kotlin or Swift code for
the fxa-client component, and see if you can generate something similar from
`fxa-client.idl`. Take a look at the way the scaffolding generator works to
see how to get started.

## What could possibly go wrong?

Lots!

The complexity of maintaining all this tooling could be a greater burden then maintaining
the manual bindings. We might isolate expertise in a small number of team members. We might
spend more time working on this tooling than we'll ever hope to get back in time savings
from the generated code.

By trying to define a one-size-fits-all API surface, we might end up with suboptimal
APIs on every platform, and it could be harder to tweak them on a platform-by-platform
basis.

The resulting autogenerated code might be a lot harder to debug when things go wrong.
