# Fork of mozilla/uniffi-rs

Fork of [mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs). The fork maintains major version
compatibility with upstream. When using matching major version of upstream and fork, its possible
to use either of them to generate either scaffolding or foreign bindings. For example, when using
`mozilla/uniffi-rs:v0.25.0` and `NordSecurity/uniffi-rs:v0.25.0-1`, its possible to use upstream to
generate Rust scaffolding, and use the fork to generate foreign bindings. The fork contains
[Github Actions code](.github/workflows/tests.yml) to run existing upstream tests. ABI compatibility
with upstream is maintained with automated tests in [fixtures/coverall-upstream-compatibility](fixtures/coverall-upstream-compatibility/Cargo.toml).

The fork exists to implement ad hoc features for our internal use, and to ease the development
process of external bindings generators. The ad hoc features are intended to eventually be
contributed back to upstream.

### Ad hoc features

Currently the only ad hoc feature is docstrings. Docstrings allow the consumer to write comments in
UDL file, and the comments are emitted in generated bindings. The comments are emitted without any
transformations. What you see in UDL is what you get in generated bindings. The only change made to
UDL comments are the comment syntax specific to each language. Docstrings can be used for most
declarations in UDL file. Docstrings are parsed as AST nodes, so incorrectly placed docstrings will
generate parse errors. Docstrings in UDL are comments prefixed with `///`. There is ongoing work to
contribute docstrings to upstream [#1493](https://github.com/mozilla/uniffi-rs/pull/1493)
[#1498](https://github.com/mozilla/uniffi-rs/pull/1498).

To use the fork to generate docstrings, you can use upstream (or fork) to generate Rust scaffolding
code, and use the fork to generate foreign bindings.

Example of docstrings in UDL file.
```java
/// The list of supported capitalization options
enum Capitalization {
    /// Lowercase, i.e. `hello, world!`
    Lower,

    /// Uppercase, i.e. `Hello, World!`
    Upper
};

namespace example {
    /// Return a greeting message, using `capitalization` for capitalization
    string hello_world(Capitalization capitalization);
}
```

### External generators

Writing external bindings generators might require small tweaks to upstream code. The tweaks are
usually trivial, and can be contributed back to upstream immediately. The problem is that even if
the changes are merged into upstream, the merged changes can't immediately be used in an external
bindings generator. That is because external bindings generators target a specific released upstream
version, not `main:HEAD`. The merged changes may only be used in an external bindings generator
after next upstream version is released. In this situation, the fork allows us to easily create
customized versions of existing upstream releases. In the future, we could try to come to an
agreement with upstream maintainers so they would patch the required changes onto existing
releases. List of external generators using this fork:

- [NordSecurity/uniffi-bindgen-cpp](https://github.com/NordSecurity/uniffi-bindgen-cpp)
- [NordSecurity/uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs)
- [NordSecurity/uniffi-bindgen-go](https://github.com/NordSecurity/uniffi-bindgen-go)

## Versioning

`NordSecurity/uniffi-rs` is versioned separately from `mozilla/uniffi-rs`. `NordSecurity/uniffi-rs`
follows the [SemVer rules from the Cargo Book](https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility)
which states "Versions are considered compatible if their left-most non-zero major/minor/patch component
is the same". A breaking change is [any modification](docs/uniffi-versioning.md) that demands the
consumer of the bindings to make corresponding changes to their code for continued functionality.

Versioning `NordSecurity/uniffi-rs` separately from `mozilla/uniffi-rs` makes room to properly
version changes made to the fork. To make the underlying uniffi-rs version obvious, the fork uses
versioning scheme `vX.Y.Z+vA.B.C`, where `X.Y.Z` is the version of the fork, and `A.B.C` is
the version of uniffi-rs it is based on.

# Branching

`main` branch contains the latest version of the code. New upstream versions are merged into `main`
from upstream release tags. Releases are tagged on `main` branch. Existing releases are patched by
creating a branch from an existing release tag. Any changes destined for an existing release must
first be merged into `main`, then cherry-picked from `main` into existing release. This ensures all
changes/fixes will be included in upcoming releases.

## Release instructions

The fork contains custom release instructions [docs/release-process-nordsec.md](docs/release-process-nordsec.md).

## Changelog

The changes made in this fork are tracked in [CHANGELOG.md](CHANGELOG.md). The original changelog
file is available at [mozilla/uniffi-rs/blob/main/CHANGELOG.md](https://github.com/mozilla/uniffi-rs/blob/main/CHANGELOG.md).
