# Navigating the code

The code for UniFFI is organized into the following crates:

- **[`./uniffi_bindgen`](./api/uniffi_bindgen/index.html):** This is the source for the `uniffi-bindgen` executable and is where
  most of the logic for the UniFFI tool lives. Its contents include:
    - **[`./uniffi_bindgen/src/interface/`](./api/uniffi_bindgen/interface/index.html):** The logic for parsing `.udl` files
      into an in-memory representation called [`ComponentInterface`](./api/uniffi_bindgen/interface/struct.ComponentInterface.html),
      from which we can generate code for different languages.
    - **[`./uniffi_bindgen/src/scaffolding`](./api/uniffi_bindgen/scaffolding/index.html):** This module turns a
      [`ComponentInterface`](./api/uniffi_bindgen/interface/struct.ComponentInterface.html) into *Rust scaffolding*, the code that
      wraps the user-provided Rust code and exposes it via a C-compatible FFI layer.
    - **[`./uniffi_bindgen/src/bindings/`](./api/uniffi_bindgen/bindings/index.html):** This module turns a
      [`ComponentInterface`](./api/uniffi_bindgen/interface/struct.ComponentInterface.html) into *foreign-language bindings*,
      the code that can load the FFI layer exposed by the scaffolding and expose it as a
      higher-level API in a target language. There is a sub-module for each supported language.
- **[`./uniffi`](./api/uniffi/index.html):** This is a run-time support crate that is used by the generated Rust scaffolding. It
  controls how values of various types are passed back-and-forth over the FFI layer, by means of the
  [`ViaFfi`](./api/uniffi/trait.ViaFfi.html) trait.
- **[`./uniffi_build`](./api/uniffi_build/index.html):** This is a small hook to run `uniffi-bindgen` from the `build.rs` script
  of a UniFFI component, in order to automatically generate the Rust scaffolding as part of its build process.
- **[`./uniffi_macros`](./api/uniffi_macros/index.html):** This contains some helper macros that UniFFI components can use to
  simplify loading the generated scaffolding, and executing foreign-language tests.
