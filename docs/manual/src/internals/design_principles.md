# Design Principles

These are some high-level points to consider when making changes to UniFFI (or when wondering why past changes were made in a particular way).


### Prioritize Mozilla's short-term needs

The initial consumers of this tool are teams working on features for Mozilla's mobile browsers.
While we try to make the tool generally useful, we'll invest first in things that are the most valuable
to those teams, which are reflected in the points below.


### Safety First

The generated bindings need to be safe by default. It should be impossible for foreign-language code
to trigger undefined behaviour in Rust by calling the public API of the generated bindings, even if it
is called in egregiously wrong or malicious ways. We will accept reduced performance in the interests
of ensuring this safety.

(The meaning of "impossible" and "public API" will of course depend on the target language. For example,
code in Python might mutate internal attributes of an object that are marked as private with a leading
underscore, and there's not much we can do to guard against that.)

Where possible, we use Rust's typesystem to encode safety guarantees. If that's not possible then the
generated Rust code may use `unsafe` and assume that the generated foreign-language code will uphold
safety guarantees at runtime.

**Example:** We insist that all object instances exposed to foreign-language code be `Sync` and `Send`,
so that they're safe to access regardless of the threading model of the calling code. We do not allow
thread-safety guarantees to be deferred to assumptions about how the code is called.

**Example:** We do not allow returning any borrowed data from function calls, because we can't make
any guarantees about when or how the foreign-language could access it.


### Performance is a feature, but not a deal-breaker

Our initial use-cases are not performance-critical, and our team are not low-level Rust experts,
so we're highly motivated to favour simplicity and maintainability over performance. Given the choice
we will pick "simple but slow" over "fast but complicated".

However, we know that performance can degrade through thousands of tiny cuts, so we'll keep iterating
towards the winning combination of "simple *and* fast" over time.

**Example:** Initial versions of the tool used opaque integer handles and explicit mutexes to manage
object references, favouring simplicity (in the "we're confident this works as intended" sense) over
performance. As we got more experience and confidence with the approach tool we replaced handles with
raw `Arc` pointers, which both simplified the code and removed some runtime overheads.

**Violation:** The tool currently passes structured data over the FFI by serializing it to a byte
buffer, favouring ease of implementation and understanding over performance. This was fine as a starting
point! However, we have not done any work to measure the performace impact or iterate towards something
with lower overhead (such as using `repr(C)` structs).


### Produce bindings that feel idiomatic for the target language

The generated bindings should feel idiomatic for their end users, and what feels idiomatic can differ
between different target languages. Ideally consumers should not even realize that they're using
bindings to Rust under the hood.

We'll accept extra complexity inside of UniFFI if it means producing bindings that are nicer for consumers to use.

**Example:** We case-convert names to match the accepted standards of the target language,
so a method named `do_the_thing` in Rust might be called `doTheThing` in its Kotlin bindings.

**Example:** Object references try to integrate with the GC of the target language, so that holding
a reference to a Rust struct feels like holding an ordinary object instance.

**Violation:** The Kotlin bindings have an explicit `destroy` method on object instances, because we haven't
yet found a good way to integrate with the JVM's GC.


### Empower users to debug and maintain the tool

To succeed long-term, we can't depend on a dedicated team of "UniFFI experts" for debugging and maintenance.
The people using the tool need to be empowered to debug, maintain and develop it.

If you're using UniFFI-generated bindings and something doesn't work quite right, it should be possible
for you to dig in to the generated foreign-language code, follow it through to the underlying Rust code,
and work out what's going wrong without being an expert in Rust or UniFFI.

**Example:** We try to include comments in the generated code to help guide users who may be reading
through it to debug some issue.

**Violation:** We don't have very good "overview" documentation on how each set of foreign-language bindings
works, so someone trying to debug the Kotlin bindings would need to poke around in the generated code to
try to build up a mental model of how it's supposed to work.

**Violation:** A lack of structure in our code-generation templates means that it's hard for non-experts
to find and change the codegen logic for a particular piece of functionality.
