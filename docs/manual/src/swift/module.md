# Compiling a Swift module

Before you can import the generated Swift bindings as a module (say, to use them
from your application, or to try them out using `swift` on the command-line) you
first need to compile them into a Swift module.

To do so, you'll need to crate a `.modulemap` file, which reassures Swift that it's
safe to expose the underlying C FFI layer. It should look something like this,
where `example` is the namespace as used in the `.udl` file:

```
module example {
    header "path/to/uniffi_example-Bridging-Header.h"
    export *
}
```

Then use `swiftc` to combine the cdylib from your Rust crate with the generated
Swift bindings:

```
swiftc
    -module-name example                      # Name for resulting Swift module
    -emit-library -o libexample.dylib         # File to link with if using Swift REPL
    -emit-module -emit-module-path ./         # Output directory for resulting module
    -parse-as-library
    -L ./target/debug/                        # Directory containing compiled Rust crate
    -lexample                                 # Name of compiled Rust crate cdylib
    -Xcc -fmodule-map-file=example.modulemap  # The modulemap file from above
    example.swift                             # The generated bindings file
```

This will produce an `example.swiftmodule` file that can be loaded by
other Swift code or used from the Swift command-line REPL.
