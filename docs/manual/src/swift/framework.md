# Rust project as individual iOS framework

Wrap the Rust crate into an iOS Framework, allowing separate modifications to the related Rust code and UniFFI-generated program configurations for easier integration management and usage in the future.

Overall, you need:

1. Generate an Xcode project file for the Rust crate and compile it into a static library.
2. Create a new iOS Framework project and import the generated target dependencies.
3. Compile UDL file to generate related Swift bindings.
4. Import the generated binding header file into the public header files of the Framework.

## Compile Rust crate using `cargo-xcode`

First, we need to install `cargo-xcode`. This tool can help us generate Xcode project files and compile them into static libraries.

Run the command `cargo install cargo-xcode` to install.

We need to modify the `Cargo.toml` file and add crate-type = ["lib", "staticlib"] in the [lib] section. Here you can add other types according to your needs, but only `staticlib` and `cdylib` can be recognized by `cargo-xcode`.

```toml
[lib]
crate-type = ["lib", "staticlib"]
```

Then run `cargo xcode`, which will generate `<rust-project-name>.xcodeproj` file.

## Create a Framework and add dependencies

Create a new iOS Framework project and drag the `<rust-project-name>.xcodeproj` mentioned above into it.

Add `<rust-project-name>-staticlib` to `Build Phases`-`Target Dependencies` in the iOS Framework.

Add `lib<rust-project-name>_static.a` to the `Link Binary With Libraries` in iOS Framework.

## Generate bindings

In the iOS Framework's `Build Rules`, add a `Run Script` to handle `*.udl` and generate the corresponding bindings.

* Add a build rule processing files with names matching *.udl.

  * Use something like the following as the custom script:
    * `$HOME/.cargo/bin/uniffi-bindgen-cli generate $INPUT_FILE_PATH --language swift --out-dir $INPUT_FILE_DIR/Generated`

  * Add both the .swift file and the generated bridging header as output files:
    * `$(INPUT_FILE_DIR)/Generated/$(INPUT_FILE_BASE).swift`
    * `$(INPUT_FILE_DIR)/Generated/$(INPUT_FILE_BASE)FFI.h`

* Add your `.udl` file to the `Compile Sources` build phase for your framework, so that Xcode will process it using the new build rule and will include the resulting outputs in the build.

You do not need to add the generated Swift code to the list of `Compile Sources` and should not attempt to compile it explicitly; Xcode will figure out what it needs to do with this code based on it being generated from the Build Rule for your `.udl` file.

## Import header files

Import the generated header file in `<framework-name>.h` of iOS Framework.

```c
#import <Foundation/Foundation.h>

//! Project version number for <framework-name>.
FOUNDATION_EXPORT double <framework-name>VersionNumber;

//! Project version string for <framework-name>.
FOUNDATION_EXPORT const unsigned char <framework-name>VersionString[];

// In this header, you should import all the public headers of your framework using statements like #import <framework-name>/PublicHeader.h>

#import "Generated/<rust-project-name>FFI.h"

```

For this to work without complaint from Xcode, you also need to add the generated header file as a Public header in the "Headers" build phase of your project (which is why it's useful to generate this file somewhere in your source tree, rather than in a temporary build directory).

## Examples

After completing the above steps, you can use your Framework by dragging it into your project and importing `<framework-name>`.

It also provides an [ios-with-framework](examples/app/ios-with-framework/) that you can check out under examples/app/ios-with-framework/.

* `ios-with-framework`: Root directory of the sample project

  * `iOS-UniFFI-Framework`: Includes handling of compiling Rust crates into static libraries and generating bindings.
  * `iOS-UniFFI-App`: Includes the use of the generated framework.

## Known issues

* If you encounter an error when generating bindings, please check if `uniffi-bindgen-cli` is installed. If the path is incorrect, please modify the script path in `Build Rules`.
