
## Release Process

We use [cargo-release](https://crates.io/crates/cargo-release) to simplify the release process.
(We rely on v0.18 or later because it has support for workspaces. Install this with
`cargo install cargo-release`, not to be confused with the different `cargo install release`!)
It's not (yet) quite an ideal fit for our workflow, but it helps! Steps:

1. Take a look over `CHANGELOG.md` and make sure the "unreleased" section accurately reflects the
   contents of the new release.
   * Anything that could cause a consumer project to behave differently if it upgraded
     to the new version, should be called out as a breaking change and should trigger
     a minor version bump (we're below `v1.0` for semver purposes).
1. Start a new branch for the release. We typically use a single branch for all point releases,
   so it should be named something like `release-v0.6.x`:
    * `git checkout -b release-v{MAJOR}.{MINOR}.x`
    * `git push -u origin release-v{MAJOR}.{MINOR}.x`
1. Run `cargo release {MAJOR}.{MINOR}.{PATCH}` to perform a dry-run and check that the things
   it is proposing to do seem reasonable.
1. Run `cargo release -x {MAJOR}.{MINOR}.{PATCH}` to perform real run and to
   bump version numbers and *publish the release to crates.io* and *create the tag*.
   It does not push the tag or branch to github, but it will publish to crates.io, so
   take care!
1. Push your branch, and make a PR to request it be merged to the main branch.
    * `git push origin --tags`
