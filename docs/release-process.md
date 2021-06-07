
## Release Process

We use [cargo-release](https://crates.io/crates/cargo-release) to simplify the release process.
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
1. Run `cargo release --dry-run -vv {MAJOR}.{MINOR}.{PATCH}` and check that the things
   it is proposing to do seem reasonable.
1. Run `cargo release {MAJOR}.{MINOR}.{PATCH}` to bump version numbers and
   publish the release to crates.io.
1. Manually update the section header in `CHANGELOG.md` following the instructions
   at the top of the file.
    * This is a limitation of using `cargo release` in a workspace, possibly related
      to [sunng87/cargo-release#222](https://github.com/sunng87/cargo-release/issues/222)
1. Run `git commit -a --amend` to include the changelog fixes and fix up the version number
   in the commit message.
    * Manually replace `{{version}}` with `v{MAJOR}.{MINOR}.{PATCH}`.
    * This is a limitation of using `cargo release` in a workspace,
      ref [sunng87/cargo-release#222](https://github.com/sunng87/cargo-release/issues/222)
1. Tag the release commit in github.
    * `git tag v{MAJOR}.{MINOR}.{PATCH}`
    * `git push origin v{MAJOR}.{MINOR}.{PATCH}`
1. Push your branch, and make a PR to request it be merged to the main branch.
    * `git push origin`
