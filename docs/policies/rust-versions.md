# Rust Versions

Like almost all Rust projects, the entire point of the UniFFI project is that
it be used by external projects. If UniFFI always uses Rust features available
in only the very latest Rust version, this will cause problems for projects
which aren't always able to be on that latest version.

Given UniFFI is currently developed and maintained by Mozilla staff, it should
be no surprise that an important consideration is mozilla-central (aka, the
main Firefox repository). This policy exists to ensure UniFFI can always be
used by mozilla-central.

## Mozilla-central Rust policies.

The Rust policy for mozilla-central has an official
[Rust Update Policy Document
](https://firefox-source-docs.mozilla.org/writing-rust-code/update-policy.html),
the tl;dr of which is:

* There's a "Uses" version which all of Mozilla's CI uses for releasing Firefox.
  Discover it in the most recent updates to [this meta bug on Bugzilla](https://bugzilla.mozilla.org/show_bug.cgi?id=1504858)

* There's a "Requires" version, which Mozilla is committed to supporting,
  primarily for downstream builders such as Linux distributions.
  [Discover it here](https://searchfox.org/mozilla-central/search?q=MINIMUM_RUST_VERSION&path=python/mozboot/mozboot/util.py). This version is tested in mozilla-central's CI,
  but nothing is shipped by mozilla with it.

# UniFFI Rust version policy

Our official Rust version policy is:

* UniFFI will ship using, have all tests passing, and have clippy emit no
  warnings, with the same version mozilla-central currently "uses".

* UniFFI must be capable of building (although not necessarily with all tests
  passing nor without clippy errors or other warnings) with the same version
  mozilla-central currently "requires".

* This policy only applies to the "major" and "minor" versions - a different
  patch level is still considered compliant with this policy.

# Updating these versions

As versions inside mozilla-central change, we will bump the UniFI versions
accordingly. While newer versions of Rust can be expected to work correctly
with our existing code, it's likely that clippy will complain in various ways
with the new version. Thus, a PR to bump the minimum version is likely to also
require a PR to make changes which keep clippy happy.

In the interests of avoiding redundant information which will inevitably
become stale, the circleci and rust-toolchain configuration links below
should be considered the canonical source of truth for the currently supported
official Rust version.

Unfortunately these versions are spread out over a few places.

## Updating the "Uses" version

* Update the version specified in [`rust-toolchain.toml`](https://github.com/mozilla/uniffi-rs/blob/main/rust-toolchain.toml)
* In [our circle CI integration](https://github.com/mozilla/uniffi-rs/blob/main/.circleci/config.yml)
  you will find a number of docker references similar to `cimg/rust:1.XX` - all of these
  should be updated; while some tasks, such as publishing the docs, are largely indepdenent of
  the requirements in this policy, we should keep them all consistent where we can.

## Updating the "Minimum Supported Version

* In [our circle CI integration](https://github.com/mozilla/uniffi-rs/blob/main/.circleci/config.yml)
  you will find a task `prepare-rust-min-version` which specifies this version via executing
  `rustup update` with the version. This is the version to update.
