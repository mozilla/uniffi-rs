# uniffi

This is a little experiment in building cross-platform components in Rust, based on things
we've learned in the [mozilla/application-services](https://github.com/mozilla/application-services)
project.

It's at the "very hand-wavy prototype" stage, so don't get your hopes up just yet ;-)

## What?

We're interested in building re-useable components for sync- and storage-related browser
functionality - things like [storing and syncing passwords](https://github.com/mozilla/application-services/tree/main/components/logins),
[working with bookmarks](https://github.com/mozilla/application-services/tree/main/components/places) and
[signing in to your Firefox Account](https://github.com/mozilla/application-services/tree/main/components/fxa-client).

We want to write the code for these components once, in Rust. We want to easily re-use these components from
all the different languages and on all the different platforms for which we build browsers, which currently
includes JavaScript for PCs, Kotlin for Android, and Swift for iOS.

And of course, we want to do this in a way that's convenient, maintainable, and difficult to mess up.

## How?

Our current approach to building shared components in Rust involves writing a lot
of boilerplate code by hand. Take the [fxa-client component](https://github.com/mozilla/application-services/tree/main/components/fxa-client)
as an example, which contains:

* The [core functionality](https://github.com/mozilla/application-services/tree/main/components/fxa-client/src) of the component, as a Rust crate.
* A second Rust crate for [the FFI layer](https://github.com/mozilla/application-services/tree/main/components/fxa-client/ffi),
  which flattens the Rust API into a set of functions and enums and opaque pointers that can be accessed from any language capable
  of binding to a C-style API.
* A [Kotlin package](https://github.com/mozilla/application-services/tree/main/components/fxa-client/android/src/main/java/mozilla/appservices/fxaclient)
  which wraps that C-style FFI layer back into rich classes and methods and so-on, for use in Android applications.
* A [Swift package](https://github.com/mozilla/application-services/tree/main/components/fxa-client/ios/FxAClient)
  which wraps that C-style FFI layer back into rich classes and methods and so-on,for use in iOS applications.
* A third Rust crate for [exposing the core functionality to JavaScript via XPCOM](https://searchfox.org/mozilla-central/source/services/fxaccounts/rust-bridge/firefox-accounts-bridge) (which doesn't go via the C-style FFI).

That's a lot of layers! We've developed some helpers to make it easier, but it's still a lot of repetitive
similarly-shaped code, and a lot of opportunities for human error.

What if we didn't have to write all of that by hand?

In an aspirational world, we could get this kind of easy cross-language interop for
free using [wasm_bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) and
[webassembly interface types](https://hacks.mozilla.org/2019/08/webassembly-interface-types/) -
imagine writing an API in Rust, annotating it with some `#[wasm_bindgen]` macros,
compiling it into a webassembly bundle, and being able to import and use that bundle
from any target language, complete with a rich high-level API!

That kind of tooling is not available to shipping applications today, but that doesn't
mean we can't take a small step in that general direction while the Rust and Wasm ecosystem
continues to evolve.

### Key Ideas

* Specify the component API using an abstract *Interface Definition Language*.
* When implementing the component:
  * Process the IDL into some Rust code scaffolding to define the FFI, data classes, etc.
  * Have the component crate `!include()` the scaffolding and fill in the implementation.
* When using the component:
  * Process the IDL to produce FFI bindings in your language of choice
  * Use some runtime helpers to hook it up to the compiled FFI from the component crate.


## Status

This is all very experimental and incomplete, but we do have some basic examples working, implementing
functions in Rust and calling them from Kotlin, Swift, and Python.
Take a look in the [`./examples/`](./examples/) directory to see them in action.

## Component Interface Definition

We'll abstractly specify the API of a component using the syntax of [WebIDL](https://en.wikipedia.org/wiki/Web_IDL),
but without getting too caught up in matching its precise semantics. This choice is largely driven by
the availability of quality tooling such as the [weedle crate](https://docs.rs/weedle/0.11.0/weedle/),
general familiarity around Mozilla, and the desire to avoid bikeshedding any new syntax.

We'll model the *semantics* of a component's API loosely on the primitives defined by the
[Wasm Interface Types](https://github.com/WebAssembly/interface-types/blob/master/proposals/interface-types/Explainer.md)
proposal (henceforth "WIT"). WIT aims to solve a very similarly-shaped problem to the one we're faced
with here, and by organizing this work around similar concepts, we might make it easier to one day
replace all of this with direct use of WIT tooling.

In the future, we may be able to generate the Interface Definition from annotations on the Rust code
(in the style of `wasm_bindgen` or perhaps the [`cxx` crate](https://github.com/dtolnay/cxx)) rather than from a separate IDL file. But it's much easier to get
started using a separate file.

The prototype implementation of parsing an IDL file into an in-memory representation of the component's
APIs is in [./src/types.rs](./src/types.rs). See [`arithmetic.idl`](./examples/arithmetic/src/arithmetic.idl)
for a simple example that actually works today, or see [`fxa-client.idl`](./examples/fxa-client/fxa-client.idl)
for an aspirational example of an interface for a real-world component.

#### Primitive Types

We'll avoid WedIDL's sparse and JS-specific types and aim to provide similar primitive types
to the WIT proposal: strings, bools, integers of various sizes and signedeness. We already know
how to pass these around through a C-style FFI and the details don't seem very remarkable.

These all pass by copying (including strings, which get copied out of Rust and into the host language
when transiting the FFI layer).

#### Functions

These are what they say on the tin - named callables that take typed arguments and return a typed result.
In WebIDL these always live in a namespace, like so:

```
namespace MyFunctions {
  my_function();
  string concat(string s1, string s2);
};
```

In the FFI, these are `extern "C"` functions that know how to convert values to and from Rust and the host
language. (WIT calls this "lifting" and "lowering" and we'll use the same terminology here).


#### Object Types (a.k.a. Reference Types, Handle Types, Structs, Classes, what-have-you)

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

In the FFI, instances are represented by an opaque `u64` handle, and their methods become `extern "C"` functions
that work just like plain functions, but take a handle as their first argument.

When generating component scaffolding, we'll rely on hand-written Rust code to provide a `MyObject` struct with
apropriate methods. we'll transparently create a HandleMap to hold instances of this struct, and a suite of
`extern "C"` functions that load handles into struct instances and delegate to their methods. Rust's strong typing
will help us ensure that the generated scaffolding code fits together properly with the core component code.

When generating language-specific bindings, these becomes a `class` or equivalent. Each instance of the class
will hold a handle to the corresponding instance on the Rust side, and its methods will call the exposed
`extern "C"` functions from the FFI layer in order to delegate operations to the Rust code.

TODO:
* Can we use member attributes to annotate which methods require mutable vs shared access?
* Can we use member attributes to identify which methods may block, and hence should be turned into a deferred/promise/whatever.

#### Record Types (a.k.a. Value Types, Data Classes, Protobuf Messages, and so-on)

These are structural types that are passed around *by value* and are typically only used for their data.
In current hand-written components, we pass these between Rust and the host language by serializing into
JSON or Protocol Buffers and deserializing on the other side.

In WebIDL this corresponds to the notion of a `dictionary`, which IMHO is not a great
name for them in the context of our work here, but will do the job:

```
dictionary MyData {
  required string foo;
  u64 value = 0;
}
```

In the WIT proposal these are "records" and we use the same name here internally.

In the FFI layer, records *do not show up explicitly*. Functions that take or return a record will do so
via an opaque byte buffer, with the calling side serializing the record into the buffer and the receiving
side deserializing it. Buffers are always freed by the host language side (using a provided destructor
function for buffers that originate from Rust).

When generating the component scaffolding, we'll turn the record description into a Rust `struct`
with appropriate fields, and helper methods for serializing/deserializing from a byte buffer.

When generating language-specific bindings, records become a "data class" or similar construct,
again with field access and serialization helpers.

Since we are autogenerating the code on both sides of serializing/deserializing records, we will
probably not use protocol buffers or JSON for this, but will instead use a simple bespoke encoding.
We assume that both producer and consumer will be build from the same IDL file using the same version
of `uniffi`. (Our current build tooling enforces this, and we'll try to build some simple hooks into
the generated code to ensure it as well).

#### Sequences

Both WebIDL and WIT have a builtin `sequence` type and we should use it verbatim.

```
interface MyObject {
    sequence<Foo> getAllTheFoos();
}
```

In current hand-written components we use ad-hoc Protobuf messages for this, e.g. the fxa-client
component has an `AccountEvent` record for a single event and an `AccountEvents` record for a list
of them. Since we're auto-generating things we'll instead use a more generic, re-useable implementation.

In the FFI layer, these operate similarly to records, passing back and forth via an opaque bytebuffer.

The generated scaffolding accepts and returns sequences as `Vec`s.

When generating language-specific bindings, sequences will show up as the native list/array/whatever type.

#### Enums

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

In the FFI layer these will be encoded into an unsigned integer type.

When generating the component scaffolding, these will become a Rust enum in the obvious fashion.

When generating language-specific bindings, these will show up however it's most obvious for an
enum to show up in that language.

There is also more sophisticated stuff in there, like union types and nullable
types. We haven't really thought about how to map those on to what we need.

#### Nullable types

Nullable types are annotated in WebIDL using a `?`. For example:

```
namespace geometry {
  Point? intersection(Line ln1, Line ln2);
};
```

In Rust, there is no `null` and thus, those values are represented by an `Option`. So, for the above example, that would be `Option<Point>`. 

In the FFI layer, nullable values will either be encoded into a single `0` byte, indicating a `null` value, or a single `1` byte followed by the non-null value.

Each binding will interpret the value as its own nullable type.

#### TODO: Union types

WebIDL has some support for these, and they're probably useful, but we haven't worked through
any details of how they might show up in a sensible way on both sides of the generated API.

#### TODO: Callbacks

WebIDL has some syntax for them, but we haven't looked at this in any detail
at all. It seems hard, but also extremely valuable because handling callbacks
across the FFI boundary has been a pain point for us in the past.

## Code Generation

Is still in its infancy, but we're working on it. The current implementation uses
[`askama`](https://docs.rs/askama/) for templating because it seems to give nice integration
with the Rust type system.

#### Scaffolding Generation

Currently a very hacky attempt in [./src/scaffolding.rs](./src/scaffolding.rs),
and a `generate_component_scaffolding(idl_file: &str)` function that's intended
to be used from the component's build file.

#### Kotlin Bindings Generation

Currently a very *very* hacky attempt in [./src/bindings/kotlin/](./src/bindings/kotlin),
and it's not yet clear exactly how we should expose this for consumers. As
something done from the component's build script? As a standlone executable
that can translate an IDL file into the bindings?

#### Swift Bindings Generation

Totally unimplemented. If you're interested in having a go at it, try copying
the Kotlin bindings generator and adapting it to your needs!

#### JS+XPCOM Bindings Generation

Totally unimplemented. If you're interested in having a go at it, try copying
the Kotlin bindings generator and adapting it to your needs!

#### Other Host Languages

We haven't even tried it yet! It could be a fun experiment to try to generate
some code that uses wasm-bindgen to expose a component to javascript.


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


## Why didn't you just use...?

#### WebAssembly ad wasm-bindgen

It would be wonderful to get much or all of this for free from wasm-bindgen, but it exclusively targets
JavaScript as a host language. The upcoming Wasm Interface Types proposal should help a lot with this,
but that's still in its early stages.

We're not aware of any production-ready WebAssembly runtimes for Android or iOS (with nice integration
with Kotlin and Swift respectively) which is a requirement for current consumers of our components.

But aspirationally, we'd be pretty happy to one day throw away much of the code in this crate in
favour of tooling from the Wasm ecosystem.

#### SWIG

SWIG is a great and venerable project in this broad domain, but it's designed for C/C++ as the
implementation language rather than Rust, and at time of writing it doesn't appear to support
generating Kotlin or Swift bindings. Either of these alone might not rule it out (e.g. we could
conceivable use time spent on `uniffi` to instead write a Kotlin backgend for SWIG) but missing them
both seems to make it a bad fit for our needs.

#### Djinni

It targets C++ as the implementation language rather than rust, and it's been explicitly put into
"maintenance only" mode by its authors.

#### Something else

Please suggest it by filing an issue! If there's existing tooling to meet our needs then you might
spoil a bit of fun, but save us a whole bunch of work!

## Code of Conduct
Please check the project's [code of conduct](./CODE_OF_CONDUCT.md)
