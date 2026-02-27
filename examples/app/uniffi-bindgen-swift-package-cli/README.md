# uniffi-bindgen-swift-package-cli

An example binary for building Swift packages based on Uniffi bindgen.

The `--build` and its related options are provided as convenience to build all required targets.

The resulting Swift package can be used as a local path dependnecy directly. Currently code
signing and notorization is out of scope for this application.

```
Usage: uniffi-bindgen-swift-package [OPTIONS] --package-name <SPEC> --out-dir <OUT_DIR>

Options:
  -p, --package-name <SPEC>
          Rust package to use for building the Swift package
          
          Cargo metadata will be searched to determine the library name.

      --library-type <LIB_TYPE>
          Type of library to package, defaults to staticlib

          Possible values:
          - staticlib: Build an embedded XCFramework with static libraries
          - dylib:     Build an embedded XCFrameowrk with embedded dynamic Framework libraries

  -o, --out-dir <OUT_DIR>
          Directory in which to generate the Swift package, i.e., Package.swift parent dir

      --swift-package-name <NAME>
          Swift package name
          
          Defaults to the package library name.

      --manifest-path <MANIFEST_PATH>
          Path to manifest for Rust workspace/package
          
          Defaults to search from current working path

  -c, --consolidate
          Consolidate crate bindings into single Swift target.
          
          Otherwise separate targets will be generated to help avoid name conflicts.

  -b, --build
          Builds package for specified targets.
          
          Otherwise assumes all targets have been built in the default target dir.

  -r, --release
          Build artifacts in release mode, with optimization
          
          Requires build flag to be set

  -F, --features <FEATURES>
          Space or comma separated list of features to activate
          
          Requires build flag to be set

      --target <TARGET>
          Target for target triple to include

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```