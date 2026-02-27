# examples-app-swift-package

Builds a Swift package, currently only static library supported, for a Uniffi Rust package including Uniffi dependencies.

## Usage

Example from `./fixtures/ext-types/lib`:

```shell
% ./run-bindgen.sh
    Finished release [optimized] target(s) in 0.28s
info: component 'rust-std' for target 'x86_64-apple-ios' is up to date
info: component 'rust-std' for target 'aarch64-apple-ios-sim' is up to date
info: component 'rust-std' for target 'aarch64-apple-ios' is up to date
info: component 'rust-std' for target 'x86_64-apple-darwin' is up to date
info: component 'rust-std' for target 'aarch64-apple-darwin' is up to date
    Finished release [optimized] target(s) in 0.15s
Creating iOS sim fat library at path '/Users/kris/Code/kriswuollett/uniffi-rs/examples/app/swift-package/../../../target/apple-ios-sim/release/libuniffi_ext_types_lib.a'
Creating macOS fat library at path '/Users/kris/Code/kriswuollett/uniffi-rs/examples/app/swift-package/../../../target/apple-macos/release/libuniffi_ext_types_lib.a'
Creating XCFramework at path '/Users/kris/Code/kriswuollett/uniffi-rs/examples/app/swift-package/../../../target/swift-package/UniffiFixtureExtTypesFFI/UniffiFixtureExtTypesFFI.xcframework'
xcframework successfully written out to: /Users/kris/Code/kriswuollett/uniffi-rs/target/swift-package/UniffiFixtureExtTypesFFI/UniffiFixtureExtTypesFFI.xcframework
Building for debugging...

...
error: emit-module command failed with exit code 1 (use -v to see invocation)
/Users/kris/Code/kriswuollett/uniffi-rs/target/swift-package/UniffiFixtureExtTypesFFI/Sources/ImportedTypesLibFFI/ImportedTypesLibFFI.swift:564:21: error: cannot find type 'UniffiOneEnum' in scope
 562 | 
 563 | public struct CombinedType: Sendable {
 564 |     public var uoe: UniffiOneEnum
     |                     `- error: cannot find type 'UniffiOneEnum' in scope
 565 |     public var uot: UniffiOneType
 566 |     public var uots: [UniffiOneType]
```
