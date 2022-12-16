
## Release Process

We use [cargo-release](https://crates.io/crates/cargo-release) to simplify the release process.
(We rely on v0.22 or later because it has support for workspaces. Install this with
`cargo install cargo-release`, not to be confused with the different `cargo install release`!)
It's not (yet) quite an ideal fit for our workflow, but it helps! Steps:

1. Take a look over `CHANGELOG.md` and make sure the "unreleased" section accurately reflects the
   contents of the new release.
   * Anything that could cause a consumer project to behave differently if it upgraded
     to the new version, should be called out as a breaking change and should trigger
     a minor version bump (we're below `v1.0` for semver purposes).

1. Identify the branch for the release. We typically use a single branch for all point releases,
   so it should be named something like `release-v0.6.x`. This also means that if you are
   making a point release, the branch will already exist.

   * If this is the first minor release, then create the branch:
      * `git checkout -b release-v{MAJOR}.{MINOR}.x`
      * `git push -u origin release-v{MAJOR}.{MINOR}.x`
   * If you are making a patch release so the branch already exists, you will need to
     merge main into it.
      * `git remote update`
      * `git checkout -b release-v{MAJOR}.{MINOR}.x --track origin/release-v{MAJOR}.{MINOR}.x`
      * `git merge --ff-only origin/release-v{MAJOR}.{MINOR}.x` # to pull in the latest on the branch
      * `git merge origin/main` # to pull in the changes you want to release

1. Run `cargo release {MAJOR}.{MINOR}.{PATCH}` to perform a dry-run and check that the things
   it is proposing to do seem reasonable. (XXX - note that the output here isn't actually
   helpful - the `cargo` output will reflect the old previous version because it will be
   building without having touched the version numbers in the `Cargo.toml`, and it doesn't
   actually propose that it's going to do anything! But it's probably worthwhile anyway!)
1. Run `cargo release -x {MAJOR}.{MINOR}.{PATCH}` to perform real run and to
   bump version numbers and *publish the release to crates.io* and *create the tag*.
   It does not push the tag or branch to github, but it will publish to crates.io, so
   take care!
1. Push your branch, and make a PR to request it be merged to the main branch.
    * `git push origin --tags`
