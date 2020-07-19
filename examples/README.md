# Example uniffi components

This directory contains some small example components implemented using uniffi. It's currently being used
more as a living test environment than user-facing docs, but hopefully it gives you a bit of an idea of
what we're up to with this crate.

Newcomers are recommended to explore them in the following order:

* [`./arithmetic/`](./arithmetic/) is the most minimal example - just some plain functions that operate
  on integers, and a simple enum.
* [`./geometry/`](./geometry/) shows how to use records and nullable types for working with more complex
  data.
* [`./sprites/`](./sprites/) shows how to work with stateful objects that have methods, in classical
  object-oriented style.
* [`./todolist`](./todolist/) Simple todolist that only adds items and shows the last item, meant to show how interacting with strings works.
* [`./fxa-client`](./fxa-client/) doesn't work yet, but it contains aspirational example of what the IDL
  might look like for an actual real-world component.
