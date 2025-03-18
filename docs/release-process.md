
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

Note that in all cases it's *not* necessary for you edit version numbers in any `Cargo.toml` file;
the commands below update all version numbers as required.

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
   * Create a release commit using: `cargo release-backend-crates {MAJOR}.{MINOR}.{PATCH}`

1. Release `uniffi`
   * **Do not execute this before the previous step.**  It depends on the published crates from that step
   * Create a release commit using: `cargo release-uniffi {MAJOR}.{MINOR}.{PATCH}` to test the `uniffi` crates.
   * Note: this will also create a tag for this version

1. Review the changes

1. Create a PR with the changes (ie, to merge your local changes on `release-v{MAJOR}.{MINOR}.x` into `origin/release-v{MAJOR}.{MINOR}.x`),
   get it approved, then merge it using the github option to create a "merge commit"

1. Publish the new release:
    * Run `cargo login` if you're not already logged in
    * Run `./tools/publish-release.sh`

1. Push the tag: `git push origin tag v{MAJOR}.{MINOR}.{PATCH}`

1. Create a PR to merge the changes back to the main branch
   * Make a new branch, eg, `git co -b merge-{MAJOR}.{MINOR}.{PATH}-to-main main`
   * On your local `main` branch, execute `git merge v{MAJOR}.{MINOR}.{PATH}`
   * Push it: `git push origin merge-{MAJOR}.{MINOR}.{PATH}-to-main`
   * Create the PR as normal, get it reviewed, merge it using the github option to create a "merge commit".

1. Publish the docs for the new version. See the [README](../manual/src/README.md) for details, but the short version is:
    * Execute `pip install -r tools/requirements_docs.txt`
    * Execute `mike deploy {MAJOR}.{MINOR} latest --update-aliases --push`
    * Wait for the deployment fairies.
    * Visit https://mozilla.github.io/uniffi-rs and make sure this new version shows
      up in the version selector and is correctly treated as the latest.

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
