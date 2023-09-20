
## Release Process

We use [cargo-release](https://crates.io/crates/cargo-release) to simplify the release process.
(We rely on v0.22 or later because it has support for workspaces. Install this with
`cargo install cargo-release`, not to be confused with the different `cargo install release`!)
It's not (yet) quite an ideal fit for our workflow, but it helps!

We use a separate version number for `uniffi` compared to all the other crates:

  - The `uniffi_*` crates are intended for either internal use or by external bindings generators.
    These crates get breaking version bumps regularly.
  - `uniffi` acts as a frontend to the other crates and re-exports the top-level UniFFI
    functionality.  `uniffi` gets breaking version bumps less often than the other crates (see below
    for details).

Steps:

1. Take a look over `CHANGELOG.md` and make sure the "unreleased" section lists all changes for
   the release.
   * Anything that affects any UniFFI consumers should be listed, this includes consumers that
     use UniFFI to generate their scaffolding/bindings, external bindings generators, etc.

1. Decide on a new version number for `uniffi` crate.  Since we are pre-`1.0`, if there are breaking
   changes then this should be a minor version bump, otherwise a patch version bump.

1. Decide on a new version number for the other `uniffi_*` crates.  Our current
   policy is to keep all of these version numbers in sync.

1. Identify the branch for the release. We typically use a single branch for all point releases,
   so it should be named something like `release-v0.6.x`. This also means that if you are making a point release, the branch
   will already exist. **The release number should match the `uniffi` version number**, not the
   version number for the `uniffi_*` crates.

   * If this is the first minor release, then create the branch:
      * `git checkout -b release-v{MAJOR}.{MINOR}.x`
      * `git push -u origin release-v{MAJOR}.{MINOR}.x`
   * If you are making a patch release so the branch already exists, you will need to
     merge main into it.
      * `git remote update`
      * `git checkout -b release-v{MAJOR}.{MINOR}.x --track origin/release-v{MAJOR}.{MINOR}.x`
      * `git merge --ff-only origin/release-v{MAJOR}.{MINOR}.x` # to pull in the latest on the branch
      * `git merge origin/main` # to pull in the changes you want to release

1. Release the backend crates
   * The first step is to release all crates other than `uniffi`
   * Test using a dry run: `cargo release-backend-crates {MAJOR}.{MINOR}.{PATCH}`
       * Note: some of the output here isn't actually helpful - the `cargo` output will reflect the old
         previous version because it will be building without having touched the version numbers in the
         `Cargo.toml`, and it doesn't actually propose that it's going to do anything! But it's probably
         worthwhile anyway!.
   * Release the crates: `cargo release-backend-crates -x {MAJOR}.{MINOR}.{PATCH}`.
       * **This will publish the new releases on crates.io**
       * This will **NOT** create a local git tag.

1. Release `uniffi`
   * **Do not execute this before the previous step.**  It depends on the published crates from that step
   * Test using a dry-run: `cargo release-uniffi {MAJOR}.{MINOR}.{PATCH}` to test the `uniffi` crates.
       * Note: some of the output here isn't actually helpful - the `cargo` output will reflect the old
         previous version because it will be building without having touched the version numbers in the
         `Cargo.toml`, and it doesn't actually propose that it's going to do anything! But it's probably
         worthwhile anyway!.
   * Release the crates: `cargo release-uniffi -x {MAJOR}.{MINOR}.{PATCH}`.
       * **This will publish the new releases on crates.io**
       * **This will create a new local git tag for the version**

1. Push your branch and tags: `git push origin --tags`
1. Make a PR to request it be merged to the main branch.

## Why avoid breaking changes for the uniffi crate?

UniFFI breaking changes are especially bad when you have a diamond-shaped dependency between uniffi,
multiple UniFFIed library crates, and an application.  We used to have this situation with Firefox:

- Firefox depended on the Glean and application-services libraries
- Both of those libraries depended on UniFFI for their bindings

This meant that whenever there was a breaking change in UniFFI, we would need to the "release
dance":
 - Release a new UniFFI version.
 - Release new Glean and application-services versions, with the new UniFFI dependency.
 - Vendor in the new Glean and application-services code into the Firefox repository.

We sometimes would need to do the release dance when the Glean and application-services weren't
really affected by the changes.  For example [this change](
https://github.com/mozilla/uniffi-rs/commit/0bf18394a49856ce0705a7eae3cb1c0127d6ffb9), which was
breaking for external bindings generators, but doesn't affect "normal" UniFFI consumers at all.

To avoid this, we came up with the following system:
 - UniFFI consumers typically only depend on the `uniffi` crate.
 - The `uniffi` crate exports the functionality from other crates needed for "normal" UniFFI
   consumers.
 - This means that breaking changes in the other crates don't need to be breaking changes for
   `uniffi`, if they don't affect the API  that `uniffi` reexports.
