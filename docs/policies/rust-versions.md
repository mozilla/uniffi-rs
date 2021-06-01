# Rust Versions

Like almost all Rust projects, the entire point of the uniffi project is that
it be used by external projects. If uniffi always uses Rust features available
in only the very latest Rust version, this will cause problems for projects
which aren't always able to be on that latest version.

Given uniffi is currently developed and maintained by Mozilla staff, it should
be no surprise that an important consideration is mozilla-central (aka, the
main Firefox repository). While at time of writing uniffi is not used by
mozilla-central, this policy is unashamedly focused on ensuring it will be
able to.

## Mozilla-central rust policies.

It should also come as no surprise that the Rust policy for mozilla-central
is somewhat flexible. There is an official [Rust Update Policy Document
](https://firefox-source-docs.mozilla.org/writing-rust-code/update-policy.html])
but everything in the future is documented as "estimated".

Ultimately though, that page defines 2 rust versions - "Uses" and "Requires",
and our policy revolves around these.

# Uniffi rust version policy

Our official rust version policy is:

* uniffi will ship using, have all tests passing, and have clippy emit no
  warnings, with the same version mozilla-central currently "uses".

* uniffi must be capable of building (although not necessarily with all tests
  passing nor without clippy errors or other warnings) with the same version
  mozilla-central currently "requires".

* This policy only applies to the "major" and "minor" versions - a different
  patch level is still considered compliant with this policy.

## Implications of this

All CI for this project will try and pin itself to this same version. At
time of writing, this means that [our circle CI integration
](https://github.com/mozilla/uniffi-rs/blob/main/.circleci/config.yml) and
[rust-toolchain configuration](https://github.com/mozilla/uniffi-rs/blob/main/rust-toolchain.toml)
will specify the version.

We should maintain CI to ensure we still build with the "Requires" version.

As versions inside mozilla-central changes, we will bump our versions
accordingly. While newer versions of Rust can be expected to work correctly
with our existing code, it's likely that clippy will complain in various ways
with the new version. Thus, a PR to bump the minimum version is likely to also
require a PR to make changes which keep clippy happy.

In the interests of avoiding redundant information which will inevitably
become stale, the circleci and rust-toolchain configuration links above
should be considered the canonical source of truth for the currently supported
official rust version.
