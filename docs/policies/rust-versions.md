# Rust Versions

Like almost all Rust projects, the entire point of the uniffi project is that
it be used by external projects. If uniffi always uses Rust features available
in only the very latest Rust version, this will cause problems for projects
which aren't always able to be on that latest version.

This is particularly important while uniffi is new and young - it's not always
possible for our policy to be something like "defer updating uniffi until your
project is on the same version of rust", because there's a good chance these
projects *need* the latest version of uniffi for it to be a useful solution.

Given uniffi is currently developed and maintained by Mozilla staff, it should
be no surprise that the elephant in the room is mozilla-central (aka, the
main Firefox repository). While at time of writing uniffi is not used by
mozilla-central, this policy is unashamedly focused on ensuring it will be
able to in the short term.

## Mozilla-central rust policies.

It should also come as no surprise that the Rust policy for mozilla-central
is somewhat flexible. There is an official [Rust Update Policy Document
](https://wiki.mozilla.org/Rust_Update_Policy_for_Firefox]) but at time of writing
is nearly 12 months out of date. There is a [meta bug to track rust version updates
](https://bugzilla.mozilla.org/show_bug.cgi?id=1504858) but that doesn't define a
policy, just tracks things as they happen.

Ultimately though, mozilla-central defines a [minimum rust version
](https://searchfox.org/mozilla-central/search?q=MINIMUM_RUST_VERSION) from which
we assume Firefox can be built.

# Uniffi rust version policy

Our official rust version policy is:

**uniffi should have all tests passing, and have clippy emit no warnings, with
the current minimum Rust version supported by mozilla-central.**

## Implications of this

All CI for this project will try and pin itself to this same version. At
time of writing, this means that [our circle CI integration
](https://github.com/mozilla/uniffi-rs/blob/main/.circleci/config.yml) will pin
itself to this same version.

As the minimum version changes, we will bump this version. While newer versions
of Rust can be expected to work correctly with our existing code, it's likely
that clippy will complain in various ways with the new version. Thus, a PR
to bump the minimum version is likely to also require a PR to make changes
which keep clippy happy.

In the interests of avoiding redundant information which will inevitably
become stale, [our circle CI integration
](https://github.com/mozilla/uniffi-rs/blob/main/.circleci/config.yml) configuration
should be considered the canonical source of truth for the currently supported
official rust version.
