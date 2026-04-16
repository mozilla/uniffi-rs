# A test for component extraction from a multi-arch library

UniFFI reads component information from shared libraries.
On macOS these can be multi-arch libraries ("fat mach"),
a single file containing libraries for multiple architectures.

This was actually broken until recently, as the file offset wasn't applied.
This test now generates a library on the fly from a minimal piece of C code.
It then runs the extraction to check it works without crashes.

Note: This test only runs on macOS, as we can't guarantee that cross-compiling for the two macOS targets actually works anywhere else.

## Run the test

```sh
cargo test
```
