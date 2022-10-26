# Build an MVP based on WebIDL and a manual workflow

* Status: accepted
* Deciders: rfkelly, linacambridge, eoger, jhugman, tarikeshaq
* Date: 2020-07-01 (or thereabouts)

## Context and Problem Statement

When [deciding to build this tool](./0000-whats-the-big-idea.md), the main risk identified was that we'd spend too
much time on an ultimately unworkable or unmaintainable idea. What early design decisions can we make to mitigate
this risk? What things are an existential risk to the success of this project that must be included in the first
version, and what things can we safely defer to future work?

In other words: how do we build an MVP of this tool that is both *minimal* and *viable*?

## Decision Drivers

* Strictly timebox our efforts to "prove out" the approach.
* Establish whether we can effecitvely maintain this kind of tool as a team.
* Support initial development of a new rust component with externally-imposed, near-term deadlines.

## Considered Options

This ADR encompasses several several related design questions, all of which feed together into an
overall approach to building the MVP of the tool:

* How will developers specify the API of their component?
  * Option A: Use an external interface definition file based on WebIDL.
  * Option B: Use an external interface definition file based on a custom language.
  * Option C: Infer the API directly from the rust code using annotations and macros.

* How will developers integrate the tool into their workflow?
  * Option D: Use build-scripts and macros to deeply integrate with the rust crate's build system.
  * Option E: Provide a tool that developers need to run by hand.

* How will we prioritize work on the capabilities offered by the tool itself?
  * Option F: Go broad, implementing many data types and API capabilities, even if they're slow or incomplete.
  * Option G: Go deep, implementing a few core data types and API capabilities, and make sure they're done well.

## Decision Outcome

Chosen options:
  * **Option A: Use an external interface definition file based on WebIDL.**
  * **Option E: Provide a tool that developers need to run by hand.**
  * **Option F: Go broad, implementing many data types and API capabilities, even if they're slow or incomplete.**

The set of options chosen here makes an explicit tradeoff, preferring to get something up and running quickly
and accepting a certain amount of jank in the developer experience. We don't have to build the perfect tool
right away, we only have to build something that's better than doing this work by hand. If we like the result
we can polish it from there.

The MVP tool will read API definitions from an external WebIDL file. This will be a bit weird and inconvenient
for consumers because WebIDL is not a precise fit for our needs, but it avoids us bikeshedding the perfect
API-definition experience during this first phase.

The MVP developer experience will involve `cargo install`ing the tool onto your system and manually running it
or integrating it into your build process. This risks being mildly inconvenient for consumers, but gives us
lots of flexibility while we learn about what a better workflow might look like.

The MVP tool may support more features than turn out to be strictly necessary, in the interests of ensuring
multiple team members can be involved in its development at this early stage. As a tradeoff, the MVP generated
code will be allowed to contain inefficiencies and limitations that hand-written code might not,
on the premise that our first consumers are not very performance-sensitive, and that there is a lot of scope for
improving these implementation details over time.

We are likely to ***revisit every single one of these choices*** if the MVP of the tool proves successful,
and will attempt to build it in such a what that they're easy to revisit.

## Pros and Cons of the Options

### Option A: Use an external interface definition file based on WebIDL.

We can require developers to specify their component API in an external definition file,
using the syntax of WebIDL to provide something that's familiar and has an existing spec.

* Good, because WebIDL exists and has the features we need for our first consumer.
* Good, because the `weedle` crate provides a ready-made parser for WebIDL.
* Good, because WebIDL has some base level of familiarity around Mozilla.
* Bad, because developers will need to duplicate their API, once in the Rust code and once in the IDL.
* Bad, because WebIDL is designed for a different use-case, so it's likely to be an awkward fit.
* Bad, because `weedle` doesn't generate particularly helpful error messages (it seems designed for
  parsing known-good WebIDL definitions rather than helping you develop new ones).

Ultimately, this seems like the lowest-cost way to get started, while deferring the
important-but-not-existentially-risky work of making an IDL experience that fits really
well with Rust code.

### Option B: Use an external interface definition file based on a custom language.

We can require developers to specify their component API in an external definition file,
using our own custom variant of an IDL syntax.

* Good, because the syntax can be custom designed to fit well with the developer's mental
  model of the generated code.
* Good, because we already have several different IDL variants in use at Mozilla (WebIDL, XPIDL),
  so our chances of building something that feels familiar are high.
* Bad, because developers will need to duplicate their API, once in the Rust code and once in the IDL.
* Bad, because we have to make up and document a whole syntax.
* Bad, because rust parsing crates such as `nom` do not seem to generate particularly helpful error messages
  by default, adding friction for the developer.

Ultimately, while a custom syntax would probably "feel" better from the consumer's perspective
than WebIDL, the costs involved are not worth that tradeoff for the MVP. Beyond the MVP we expect
that direct annotation of the Rust code will provide a better developer experience, leaving this
option as an unnecessary middle-ground.

### Option C: Infer the API directly from the rust code using annotations and macros.

We can allow developers to sprinkle some macro annotations directly on their rust code
in order to declare the component API, similar to the approach taken by `wasm-bindgen`.

* Good, the Rust code is a single source of truth for the API definition.
* Good, because developers are familiar with this approach from other tools.
* Bad, because our team doesn't have much experience working with macros at scale.
* Bad, because from a poke around in the `wasm-bindgen` code, they seem to need to do
  some pretty scary things in order to make the macros Just Work in various edge-cases.

Ultimately, this feels like a good approach longer-term, but risks being too much of a time-sink
for the MVP.

### Option D: Use build-scripts and macros to deeply integrate with the rust crate's build system.

We can encourage consumers to structure their rust component as a crate, take a build-time dependency
on our tool, and magic things into existence as part of `cargo build`.

* Good, because it's a slick developer experience if we can make it work.
* Bad, because it assumes many details of how the consuming component is being built and deployed,
  and we don't know exactly how that will work yet.
* Bad, because it could be hard to integrate with e.g. a gradle-based build system for android packages.
* Bad, because build scripts aren't supposed to create files outside of the rust target directory,
  but it doesn't realy make sense to generate foreign language bindings into that directory.

Ultimately, this approach does not provide enough flexibility for initial consumers, risking them
declaring it a bad fit based on non-essential details of the tool itself.

### Option E: Provide a tool that developers need to run by hand.

We can provide consumers with a `uniffi-bindgen` command-line tool that they manually run on their
component code in order to generate foreign language bindings.

* Good, because gives the consumer lots of flexibility and when and where to generate the different
  bits of code.
* Bad, because consumers have to install an external tool.

Ultimately, this approach wins based on flexibility. We may also provide a light wrapper around the
tool that integrates it with `cargo build` for convenience.

### Option F: Go broad, implementing many data types and API capabilities, even if they're slow or incomplete.

We can focus our initial efforts on fleshing out a broad suite of data types and API capabilities,
spending less time focused on performance or edge-cases in the generated code.

* Good, because consumers can iterate their API with less chance of being limited by the tool.
* Good, because it makes more opportunities for team members to get involved in implementing features
  of the tool itself, helping us understand what it will be like to maintain it over time.
* Bad, because sub-optimal performance may be offputting for consumers.
* Bad, because we might accidentally entrench design decisions that limit our ability to improve
  the generated code in future.

Ultimately, this option wins based on the known needs of our first target consumer (which favours
iteration over performance) and the of team itself (which wants to ensure multiple developers are
familiar with the tool's codebase as it gets off the ground).

### Option G: Go deep, implementing a few core data types and API capabilities, and make sure they're done well.

We can focus our initial efforts on identifying just the data types and API capabilities required by our
first target consumer and implementing them really well, spending less time on features that are unlikely
to be required by the consumer.

* Good, because it shows the resulting generated code in the best possible light.
* Bad, because we might not identify the correct set of features.
* Bad, because it's harder to parallelize this kind of work among multiple team members.

Ultimately, the needs of our first target consumer make the "performance" argument fairly weak,
so this option was not selected.

## Links

* [The WebIDL specification](https://heycam.github.io/webidl/), for reference.
