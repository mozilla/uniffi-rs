# Example uniffi components

This directory contains some small example components implemented using uniffi. It's currently being used
more as a living test environment than user-facing docs, but hopefully it gives you a bit of an idea of
what we're up to with this crate.

Newcomers are recommended to explore them in the following order:

* [`./arithmetic/`](./arithmetc/) is the most minimal example - just some plain functions that operate
  on integers, and a simple enum.
* [`./geometry/`](./geometry/) shows how to use records and nullable types for working with more complex
  data.
* [`./fxa-client`](./fxa-client/) doesn't work yet, but it contains aspirational example of what the IDL
  might look like for an actual real-world component.
