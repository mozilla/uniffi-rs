# Write and maintain a custom tool for generating foreign-language bindings to rust code.

* Status: accepted
* Deciders: rfkelly, linacambridge, eoger, thomcc
* Date: 2020-07-01 (or thereabouts)

## Context and Problem Statement

On the Application Services team, we have successfully built several re-useable components for sync- and storage-related browser
functionality by following what we've dubbed the "rust-components" approach: write the bulk of the code in rust
so we can cross-compile it for different target platforms, have it expose a C-compatible FFI layer, then write a small amount
of FFI bindings code to expose the functionality to each of several different target languages (e.g. Swift and Kotlin).

The FFI layer and foreign-language bindings code is currently written by hand, a tedious and potentially error-prone
process.

Given that we expect to build additional components in this style in the future, and expect more teams at Mozilla to
do the same, can we increase the efficiency and reliability of this work by auto-generating some of this code?

## Decision Drivers

* Reduce time taken to launch a new rust component.
* Improve maintainability of existing rust components.
* Reduce possibility of errors in hand-written foreign language bindings code.
* Continue shipping components on a regular cadence.

## Considered Options

* Option A: Continue writing the FFI and foreign-language parts of rust components by hand
* Option B: Move to using WebAssembly and wasm-bindgen
* Option C: Use SWIG, Djinni, or another existing bindings-generator tool
* Option D: Write and maintain a custom tool that automates our current best practices

## Decision Outcome

Chosen option:
  * **Option D: Write and maintain a custom tool that automates our current best practices**

On balance, this option provides us with the best tradeoff of potential upside and the ability to limit downside.
If the approach succeeds then we expect to realize significant improvement in maintenance costs of rust-components
code by reducing boilerplate and human error. Building our own will involve the least up-front investment before
we can start to show results, because we did not identify any existing tools that were a close-enough fit for our needs.
The first versions of the tool don't have to be perfect, or even particularly *good* - they just have to have a
better value-proposition than writing the generated code by hand.

We accept the risk that writing our own tool for this may turn out to be much more complex than expected, and
will mitigate it by aggressively time-boxing initial prototypes, by developing it in parallel with a real shipping
consumer with real deadlines, and by regularly asking the hard questions about whether the approach is working out.

## Pros and Cons of the Options

### Option A: Continue writing the FFI and foreign-language parts of rust components by hand

We could decide that hand-writing some `pub extern "C"` function wrappers and a bit of custom Swift,
Kotlin, etc isn't all that bad, and that the cost of doing so is unlikely to be offset by an investment
in more automated tooling.

* Good, because we can dedicate more people to building new component functionality, rather than working on tooling.
* Good, because each component can use whatever bespoke FFI details work best for its use-case, rather than
  taking a one-size-fits-all approach.
* Good, because we don't have to learn a new tool or maintain an existing one.

* Bad, because the time commitment for maintaining bindings will only grow as we build more components.
* Bad, because it's easy to make mistakes when writing the bindings by hand, and it has proven hard to avoid
  making similar mistakes multiple times.
* Bad, because hand-writing bindings is low-engagement work that risks feeling like a chore, and we have plenty
  of other chores already.

Ultimately, it feels like the potential long-term cost savings of automation will be significant,
but we are glad to continue having this option as a Plan B.

### Option B: Move to using WebAssembly and wasm-bindgen

The approach we've taken with rust components has many similarites to WebAssembly, particularly the
[WebAssembly Interface Types](https://hacks.mozilla.org/2019/08/webassembly-interface-types/) proposal.
We could try to use [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) to automatically generating
bindings to our rust code, and rely on the portable nature of WebAssembly to run on multiple platforms.

* Good, because the tooling around `wasm-bindgen` seems quite sophisticated and mature.
* Good, because this toolchain is maintained by folks for whom it is a full-time job.
* Good, because it meshes well with technical projects that are strategically important for Mozilla.

* Bad, because `wasm-bindgen` currently only supports JavaScript as a target language.
* Bad, because we have not been able to identify a mature solution for running WebAssembly on Android or iOS,
  which are important target platforms.
* Bad, because this is a significant departure from how we've written components in the past, which adds
  timeline risk to shipping this solution.

Ultimately, we can imagine a world in which the WebAssembly ecosystem is sufficently advanced to
make this the most compelling option, but that world seems far enough away that it doesn't make
sense for us to pursue this right now.

### Option C: Use SWIG, Djinni, or another existing bindings-generator tool

Writing code in a systems language and generating bindings for high-level languages is not a new idea.
Among existing tools in this general space are [SWIG](http://www.swig.org/) and
[Djinni](https://github.com/dropbox/djinni). We could adopt one of these existing tools instead
of inventing our own thing.

* Good, because these tools already exist and are mature, saving us development and maintenance effort.
* Good, because it would realize the goal of avoiding hand-written boilerplate.
* SWIG:
  * Bad, because it doesn't appear to have support for generating Kotlin or Swift bindings, which
    are key languages for our use-case.
  * Bad, because it is designed for C/C++ rather than Rust, meaning an unknown amount of exploratory
    work required to integrate it with our approach before we can ship anything.
* Djinni:
  * Good, because it targets several of our key languages/platforms.
  * Bad, because it is designed using C++ as the implementation language rather than Rust, meaning
    an unknown amount of exploratory work required to integrate it with our approach before we can
    ship anything.
  * Bad, because it has explicitly been put into "maintenance mode" by its authors.

Ultimately, while we could probably make one of these tools work, we could not find one that was
a close enough fit for our needs to avoid the "unknown amount of exploratory integration work" problem.
We're not willing to put off shipping incremental progress for long enough to be confident of
making the integration work.

### Option D: Write and maintain a custom tool that automates our current best practices

We can take the patterns we've established for writting the FFI layer and foreign language bindings
by hand, and encapsulate them in a custom tool to automatically generate similar code.

* Good, because it would realize the goal of avoiding hand-written boilerplate.
* Good, because the first version of the tool only has to be good enough for our limited needs,
  meaning we can defer some complexity until after we've proven out the idea.
* Good, because it can be designed up-front to meet some of our unusual needs (integrating with
  Firefox Desktop code, being built as part of a larger shared library, etc).
* Good, because we should be able to determine whether the approach is working within a fairly
  strict timebox, and fall back to hand-written bindings if required.
* Bad, because we take on all the development and maintenance burden of the tool, reducing time
  that can be spent on product features.
* Bad, because we risk isolating knowledge of how to tool works in a small number of people.
* Bad, because we might spend more time on developing and maintaining the tool than we'd ever hope
  to save from the generated code.
* Bad, because the bindings will be limited to a one-size-fits-all, lowest-common-denominator
  feature set.
* Bad, because the generated code risks being much harder to debug than hand-written bindings,
  especially while the tool itself is under heavy development.

Ultimately, while there are risks with this approach, they seem sufficiently well-understood and
well-bounded that we can try out the approach, and fall back to hand-written bindings if it doesn't
seem to be working out.

## Links

* [Engineering Program Review: Sync & Storage Components](https://docs.google.com/document/d/10I8MD_narf3D7w1F0rciye-cRpwKO6ItE_UdCtFKTmA);
  an earlier technical review of the "rust components" approach, including some discussion of the
  pain-points around manually writing FFI bindings.
* [The (not so) hidden cost of sharing code between iOS and Android](https://dropbox.tech/mobile/the-not-so-hidden-cost-of-sharing-code-between-ios-and-android);
  a kind of technical post-mortem from Dropbox exploring why they abandoned a code-sharing approach
  that is similarly shaped to the one we're pursuing with rust components. Many of the risks highlighted
  in this post also apply to our chosen solution.
