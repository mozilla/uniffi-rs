# Comparing UniFFI with Diplomat

[Diplomat](https://github.com/rust-diplomat/diplomat/) and [UniFFI](https://github.com/mozilla/uniffi-rs/)
are both tools which expose a rust implemented API over an FFI.
At face value, these tools are solving the exact same problem, but their approach
is significantly different.

This document attempts to describe these different approaches and discuss the pros and cons of each.
It's not going to try and declare one better than the other, but instead just note how they differ.
If you are reading this hoping to find an answer to "what one should I use?", then that's easy -
each tool currently supports a unique set of foreign language bindings, so the tool you should
use is the one that supports the languages you care about!

(There may even be a future where these 2 tools converge - that seems like a lot of work, but
might also provide a large payoff - more on this later)

Disclaimer: This document was written by one of the UniFFI developers, who has never used
diplomat in anger. Please feel free to open PRs if anything here misrepresents diplomat.

# The type systems

The key different between these 2 tools is the "type system". While both are exposing Rust
code (which obviously comes with its own type system), the foreign bindings need to know
lots of details about all the types expressed by the tool.

For the sake of this document, we will use the term "type universe" to define the set of
all types known by each of the tools. Both of these tools build their own "type universe" then
use that to generate both Rust code and foreign bindings.

## UniFFI's type universe
UniFFI's model is to parse an external ffi description from a `.udl` file which describes the
entire "type universe". This type universe is then used to generate both the Rust scaffolding
(on disk as a `.rs` file) and the foreign bindings.

**What's good about this** is that the entire type system is known when generating both the rust code
and the foreign binding.

**What's bad about this** is that the external UDL is very ugly and redundant in terms of the
implemented rust API.

## Diplomat's type universe

Diplomat defines its "type universe" (ie, the external ffi) using macros.

**What's good about this** is that the "ffi module" defines the canonical API and it is defined in
terms of Rust types - the redundant UDL is removed. The Rust scaffolding can also be generated
by the macros, meaning there are no generated `.rs` files involved.

Ryan even tried this for UniFFI in [#416](https://github.com/mozilla/uniffi-rs/pull/416) - but we
struck **what's bad about this**: the context in which the macro runs doesn't know about types defined
outside of that macro, which are what we need to expose.

## Limitations in the macro approach

Let's look at diplomat's simple example:

```rust
#[diplomat::bridge]
mod ffi {
    pub struct MyFFIType {
        pub a: i32,
        pub b: bool,
    }

    impl MyFFIType {
        pub fn create() -> MyFFIType { ... }
        ...
    }
}
```

This works fine, but starts to come unstuck if you want the types defined somewhere else. In this trivial example, something like:

```Rust
pub struct MyFFIType {
    pub a: i32,
    pub b: bool,
}

#[diplomat::bridge]
mod ffi {
    impl MyFFIType {
        pub fn create() -> MyFFIType { ... }
        ...
    }
}
```

fails - diplomat can't handle this scenario - in the same way and for the same reasons that Ryan's
[#416](https://github.com/mozilla/uniffi-rs/pull/416) can't - the contents of the struct aren't known.

From the Rust side of the world, this is probably solvable by sprinkling more macros around - eg, something like:

```Rust
#[uniffi::magic]
pub struct MyFFIType {
    pub a: i32,
    pub b: bool,
}
```

Might be enough for the generation of the Rust scaffolding. However, the problems are in the foreign bindings.

## How the type universe is constructed for the macro approach.

In both diplomat and [#416](https://github.com/mozilla/uniffi-rs/pull/416), the approach taken
is that the generation process wants a path to the Rust source file that contains the module in
question - in the example above, the `ffi` module annotated with `#[diplomat:bridge]`. They both
use the `syn` crate to parse the Rust code inside this module, build their type universe, then
generate the foreign bindings.

In our problematic example above, this process never sees the layout of the `MyFFIType` struct,
and nor does it see any macros annotating them.

For this approach to work, it would be necessary for this process to compile the entire crate,
including depedent crates - the actual definition of all the types might appear anywhere.
Not only would this be slow, it's not clear it could be made to work - it might be reasonable to
have constraints on what can appear in just the `ffi` mod, but if we started adding constraints
to the entire crate, the tool would become far less useful.

This is the exact same problem which caused us to decide to stop working on
[#416](https://github.com/mozilla/uniffi-rs/pull/416) - the current world where the type universe
is described externally doesn't have this problem - only the UDL file needs to be parsed when
generating the foreign bindings - Rust code isn't considered. The application-services team has
concluded that none of our non-trival use-cases for UniFFI could be described using macros,
so supporting both mechanisms is pain for no gain.

As noted in #416, `wasm-bindgen` has a similarly shaped problem, and solves it by having
the Rust macro arrange for the resulting library to have an extra data section with the
serialized "type universe" - foreign binding generation would then read this information from the
already built binary. This sounds more complex than the UniFFI team has appetite for at
the current time.

## Is this a problem for users of diplomat? Will diplomat solve it?

I couldn't find real examples using diplomat, so it's difficult to know if this
is a problem in practice. UniFFI came from a world where we had Rust crates and
a hand-written FFI that exposed types from all over the crate. If these tools
had started with the limitations from the macro approach in mind, it's possible
a different, acceptable design might have been made to work. Maybe duplicating
some structs and supplying suitable `Into` implementations might make things workable?

Diplomat comes from a very smart team. They may well come up with a novel solution, so
UniFFI should track the progress of that project to see what we can gleefully steal
in the future. As discussed below, a kind of "hybrid" approach might even be possible.

# Looking forward

Before looking forward, let's step back a little - both UniFFI and diplomat are solving the exact
same use-cases, just using a different approach to defining the type universe.
But if we ignore that, the tools take the same basic approach - they all build the
type universe, then use the representation of this type universe to define both Rust
and foreign bindings.

The type universe described by diplomat is somewhat "leaner" than that described by UniFFI -
Rust types are the first-class citizens in the universe. UniFFI defines an external type model -
for example, there's a `Type` enum where, for example, `Type::Record(Record)` represents a
Rust struct. In other words, diplomat's type world can not be divorced from Rust,
whereas UniFFI's already is.

That said though, there might be a future where merging or otherwise creating some
interoperability between these type universes might make sense. You could imagine
a world where you can use diplomat to describe your type universe, but use UniFFI's foreign
generation code to generate the Kotlin bindings. Similarly, a world where you use UniFFI
and UDL files to describe your type universe, but then use diplomat to generate
the NodeJS bindings.

Or to put it another way, you could imagine a world where both tools are split into a
"describe the type universe" portion and a "build the bindings" portion, and these tools
could be used together.

Sadly, that looks like alot of work, so someone would probably need to find a compelling
actual use-case to perform this work.

# Next steps for UniFFI

As much as some of the UniFFI team dislike the external UDL file, there's no clear path to
moving away from it. The macro approach is too limiting, and no other promising opportunities
have presented themselves. There's no clear alternative to UDL which allows a complex
type universe to be described, and at this stage, any replacement would need to be
compelling enough to make a change worthwhile, which is hard to imagine.

In the short term, the best we can probably do is to enumerate the perceived problems
with the UDL file and try to make them more ergonomic - for example, avoiding repetition of
`[Throws=SomeError]` would remove alot of noise, and some strategy for generating
documentation might go a long way.
