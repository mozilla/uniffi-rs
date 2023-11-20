# Release process

The release process is different from `mozilla/uniffi-rs`. Fork is not published to [crates.io](https://crates.io).
`cargo-release` is needed to make a release.
```
cargo install cargo-release
```

- Following [versioning semver rules](../README.md#versioning), select the next version, e.g:
    ```
    0.2.0+v0.25.0
    ```

- Create a new branch for PR.
    ```
    git checkout -b bump-X.Y.Z+vA.B.C
    ```

- Update changelog in [README.md](../README.md) to include the changes that were made since last
    version.

- Update version numbers in `Cargo.toml` files, creates a commit.
    ```
    cargo release-backend-crates --no-publish --no-tag --execute X.Y.Z+vA.B.C
    cargo release-uniffi --no-publish --no-tag --execute X.Y.Z+vA.B.C
    ```

- Push the branch and make a PR.
    ```
    git push --set-upstream origin bump-X.Y.Z+vA.B.C
    ```

- Create a tag once the PR is approved and merged. Use either Github GUI or command line. 
    ```
    git checkout main
    git pull
    git tag X.Y.Z+vA.B.C
    git push origin X.Y.Z+vA.B.C
    ```
