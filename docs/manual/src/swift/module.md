# Compiling a Swift module

Before you can import the generated Swift bindings as a module (say, to use them
from your application, or to try them out using `swift` on the command-line) you
first need to compile them into a Swift module.

To do so, you'll need both the generated `.swift` file and the corresponding
`.modulemap` file, which tells Swift how to expose the underlying C FFI layer.
Use `swiftc` to combine the cdylib from your Rust crate with the generated
Swift bindings:

```
swiftc
    -module-name example                         # Name for resulting Swift module
    -emit-library -o libexample.dylib            # File to link with if using Swift REPL
    -emit-module -emit-module-path ./            # Output directory for resulting module
    -parse-as-library
    -L ./target/debug/                           # Directory containing compiled Rust crate
    -lexample                                    # Name of compiled Rust crate cdylib
    -Xcc -fmodule-map-file=exampleFFI.modulemap  # The modulemap file from above
    example.swift                                # The generated bindings file
```

This will produce an `example.swiftmodule` file that can be loaded by
other Swift code or used from the Swift command-line REPL.

If you are creating an XCFramework with this code, make sure to rename the modulemap file
to `module.modulemap`, the default value expected by Clang and XCFrameworks for exposing
the C FFI library to Swift.
