# How to prevent version mismatches

* Status: proposed
* Date: 2021-04-01

## External bindings generators

The ADR from [#1150](https://github.com/mozilla/uniffi-rs/pull/1150) proposes moving bindings generators to a separate
crate.  This means that instead of having a single `uniffi-bindgen` binary to generate bindings, there will be a set of
`uniffi-bindgen-[language]` binaries, one for each language needed. This ADR assumes that this plan will eventually be
adopted.

## Context and Problem Statement

UniFFI is made up of several components whose versions need to be kept in sync to work properly:

  - `uniffi` the runtime dependency used by the scaffolding code
  - `uniffi-bindgen` used to generate the scaffolding code
  - `uniffi-bindgen-[language]` used to generate the bindings code

If these get out of sync we could have compatibility issues between the generated bindings and scaffolding code.  For
example, if we updated `uniffi` and modified the layout of `RustBuffer`, but then used scaffolding or bindings code that
used the old layout, this would result in undefined behavior.

To prevent the undefined behavior, we use the current version string to generate the `ComponentIterface` hash and
include that hash in the FFI function names. This means if there is a version mismatch, the user will see a linker error
or a runtime error when trying to open the dynamic library.  These errors are preferable to undefined behavior, however
they are not very developer friendly.  Also, this introduces the strict requirement that versions must match exactly
even though changes that would cause the undefined behavior are relatively rare.  See
[#1190](https://github.com/mozilla/uniffi-rs/issues/1190) for an example of this causing developer confusion and slowing
down their workflow.

This issue is compounded by the fact that versions are managed in different ways:

  - The `Cargo.toml` file of the UniFFI component
  - The `uniffi-bindgen` binary used to generate the scaffolding, which is installed in multiple places:
     - The `uniffi-bindgen` devs install on their system
     - The `uniffi-bindgen` that's installed in CI
     - The `uniffi-bindgen` that downstream consumers install on their system
  - For each binding language, the `uniffi-bindgen-[language]` used to generate the bindings.  Like `uniffi-bindgen`
    this is also install in multiple places.

Moving to separate bindings crates means the problem of version compatibility becomes more complex since we can't
realistically synchronize releases between UniFFI and all external bindings crates:

  - Both UniFFI crates and the external bindings crates should able to make a bugfix release without needing to make a
    coresponding release for all the other crates.
  - Even releases with breaking changes don't always require corresponding releases in the other crates.  For example,
    if we could update how `uniffi-bindgen-kotlin` generates the class hierarchy for Kotlin errors, without needing to
    change `uniffi` or `uniffi-bindgen`.

This means that our current method of using exact string matching for versions is no longer going to work.

Finally, there can only be one `uniffi-bindgen` and `uniffi-bindgen-[language]`, installed system wide.  This makes it
difficult for a developer to work on crates that depend on different UniFFI versions.

We've built a decent solution for the scaffolding code: [the builtin-bingen
feature](https://mozilla.github.io/uniffi-rs/tutorial/Rust_scaffolding.html#avoiding-version-mismatches-between-uniffi-core-and-uniffi-bindgen).
However, there's no similar solution for the bindings code.

application-services has a custom solution for the bindings generation.  It contains the
[embedded-uniffi-bindgen](https://github.com/mozilla/application-services/tree/main/tools/embedded-uniffi-bindgen) crate
which contains very
[simple binary](https://github.com/mozilla/application-services/blob/main/tools/embedded-uniffi-bindgen/src/main.rs)
that just runs `uniffi-bindgen`. This allows us to control the `uniffi-bindgen` version using the `Cargo.toml` for this
crate, rather than relying on the system installed version.

### Breaking changes to the UDL syntax

A related issue is breaking changes to the UDL syntax, for example the change the renamed `[Wrapped]` to `[Custom]`.
These kind of changes can lead to challenges with scaffolding and bindings generation.  For example, users could get
into a situtation where `uniffi-bindgen` can parse a UDL file, but `uniffi-bindgen-kotlin` can't.

However, this issue seems less important since a) the errors are easier to diagnose and fix and b) we have historically
only introduced breaking changes like this for "experimental" features like custom/wrapped types.

This ADR will ignore this issue and only focus on compatibility between the generated scaffolding and bindings code.

## Decision Drivers

- UniFFI should have a friendly developer experience.
  - If possible, developers shouldn't have to worry or know about this issue.
  - If that's not possible, then it should be easy for them to identify and fix the issue.
- We need to support external bindings generators and allow those projects to
  release new versions without coordinating with the UniFFI team.

## Considered Options

### **[Requirement] More flexible versioning**

As mentioned in the context, once we have external bindings generators we can't rely on simply comparing its version
string to `uniffi`.  A more flexible versioning policy is a requirement for all other options discussed here.

I think we can use semver for this:
  - The major version is incremented whenever there's a change that would break compatibility between the bindings and
    scaffolding generation code. Changing the layout of `RustBuffer` or `RustCallStatus` would be a breaking change. Our
    custom/external types work would not.
  - For code generators: the minor version is incremented whenever there's a breaking change between the generated code
    and the consumer code.  Some examples would be: changing Kotlin classes from open to closed and changing the
    argument names on Swift.
  - For `uniffi` itself and other crates, I'm not sure what a minor version change should mean.  Maybe we just don't
    bump these?
  - The patch version is incremented for non-breaking, bugfix, releases.

This means that `uniffi-bindgen` and all external bindgen tools are compatible with `uniffi` if their major versions
match.  We would update `ComponentInterface` so it only uses the major version number when calculating it's hash.

Pros:
  - Seems like a hard requirement if we want external bindings tools
  - Would significantly reduce the chance of running into a version mismatch.
  - Makes it easier to detect version mismatches when parsing `Cargo.toml`. Right now we need to determine the exact
    version that was resolved and stored in `Cargo.lock`.  If we only needed to compare the major versions, then we
    could use the specification from `Cargo.toml` directly.

Cons
  - We would have to be more careful with breaking changes. If we realized a change would break the generated code
    compatibility, then that change might need to wait before releasing it. If we didn't realize a change would break
    the generated code compatilibity and released the change anyway, then we've introduced undefined behavior in all
    consumer code.
  - Doesn't fully solve the issue of version mismatches. We would need to combine this with at one or more options
    below.

### **[Option 1] generate bindings in `build.rs`**

As mentioned in the context, we handle this issue for scaffolding by using the `builtin-bindgen` feature of
`uniffi_build`.  If we also generated the bindings code in `build.rs`, we could use a similar system:

- The Rust crate adds a dependency to each bindings generator they want to use (`uniffi-bindgen-kotlin = 1.10.1`,
  `uniffi-bindgen-swift = 1.4.0`, etc).
- Each bindings generator defines a `generate_bindings()` function, which generates the bindings and writes them to the
  `target/[language]` directory.
- We call this in `build.rs`:

```
fn main() {
  uniffi_bindgen_language1.generate_bindings();
  uniffi_bindgen_language2.generate_bindings();
  ...
}
```

- The build scripts would then copy the bindings from that directory to the final location.

One issue here is that different library consumers often want different sets of bindings generated.  For
application-services, FF IOS wants the Swift bindings generated, FF Android wants the Kotlin bindings, FF Desktop wants
the JS bindings generated, and a hypothetical 3rd-party might want Ruby bindings.  To support this each library would
add a feature gate for each language that controls which `uniffi-bindgen-[language]` crates to depend on and which
bindings to generate.

Pros:
  - Doesn't rely on a system-wide install of `uniffi-bindgen`.
  - Bindgen tool versions are specified in the `Cargo.toml`, alongside the `uniffi` version.

Cons:
  - Forces a specific order to the build process.  If you try building the bindings before the Rust component, it would
    fail since the source files haven't been generated yet (or worse, succeed with stale code).
  - Requires libraries to define a features for each language that their consumers can use.

### **[Option 2] Tell users to create embedded bindgen crates**

We could make the `application-services` solution the recommended system:

- Require each bindgen tool to expose a `clap_command()` function that returns a `clap::Command` instance.
- Create a `uniffi_bindgen::run_bindgen_app` function that executes a CLI app from a list of `clap::Command` instances
  from bindgen tools.  Each one of these commands would become a clap subcommand.
- Tell UniFFI users to create a new crate in their workspace with for their bindgen binary.  The code would look
  something like this:

```
fn main() {
    uniffi_bindgen::run_bindgen_app(vec![
       uniffi_bindgen_language1::clap_command(),
       uniffi_bindgen_language2::clap_command(),
       uniffi_bindgen_language3::clap_command(),
       ...
    ])
}
```

Support for each languages could be behind feature gates, as in `[1]`.  However, libraries could also choose to define a
preferred set of languages and require that consumers create their own bindgen crates if they want a different set. For
example, application-services might have a bindgen crate that supports Kotlin and Swift and moz-central could consume
the application-services components, but use a bindgen crate that supports JS.

Pros:
  - Doesn't rely on a system-wide install of `uniffi-bindgen`.
  - Bindgen tool versions are specified in a `Cargo.toml`.
  - Doesn't require building the main library to generate bindings.
  - Allows library consumers to define their own bindgen crates.

Cons:
  - Extra boilerplate to create the bindgen crate.
  - Bindgen tool versions are specified in a different `Cargo.toml` than where the `uniffi` version is specified.
  - Some libraries don't use workspaces, we would be forcing them to switch to using workspaces.  Note though, that
    users wouldn't have to drastically change their project layout.  They could keep their existing crate in the same
    place and make it a [root package](https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package).
  - If library consumers wants

### **[Option 3] Tool to manage UniFFI installed versions**

We create a tool that manages the bindgen binaries ourselves.

  - Users specify their bindgen tool version in the `uniffi.toml` bindings config
  - We update `uniffi-bindgen` to run the bindgen tool specified in `uniffi.toml`.  `uniffi-bindgen generate [language]`
    would:
    - Lookup the bindgen tool version from `uniffi.toml` for `[language]`
    - Installs that version of the tool inside the `target` directory ( `cargo install --root target/uniffi --version
      X.Y.Z uniffi_bindgen_[language]`). We should also store the installed versions and avoid trying to re-install if
      the version hasn't changed
    - Runs `target/uniffi/uniffi_bindgen_language` and passes it all additional args from the command line.


Pros:
  - Doesn't rely on a system-wide install of `uniffi-bindgen`.
  - Doesn't require building the main library to generate bindings.
  - Doesn't require any additional crates.

Cons:
  - Requires the most amount of work on our part.
  - Specifying versions in `uniffi.toml` as well as `Cargo.toml` could be confusing.
  - If a library consumer wanted to generate bindings for a language that wasn't specified in `uniffi.toml`, then they
    would need to manage the installed binary themselves.

## Decision Outcome

TODO

### Positive Consequences

TODO

### Negative Consequences
TODO
