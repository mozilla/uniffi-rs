# Navigating the code

The code for UniFFI is organized into the following crates:

- **`./uniffi`** ([docs.rs](https://docs.rs/uniffi/latest), [source](https://github.com/mozilla/uniffi-rs/tree/main/uniffi)): The main entry-point to UniFFI - almost all consumers should be able to use facilities expored by this module.

- **`./uniffi_bindgen`** ([docs.rs](https://docs.rs/uniffi_bindgen/latest), [source](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_bindgen)): The the source for the `uniffi-bindgen` executable and is where
  most of the logic for the foreign bindings generation lives. Its contents include:
    - **[`./uniffi_bindgen/src/interface/`](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_bindgen/src/interface):** Converting `uniffi_meta` types into an in-memory representation called [`ComponentInterface`](https://docs.rs/uniffi_bindgen/latest/uniffi_bindgen/interface/struct.ComponentInterface.html),
      from which we can generate code for different languages.
    - **[`./uniffi_bindgen/src/scaffolding`](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_bindgen/src/scaffolding):** This module generates `.rs` files for the parts of the 
      [`ComponentInterface`](https://docs.rs/uniffi_bindgen/latest/uniffi_bindgen/interface/struct.ComponentInterface.html) defined in UDL
    - **[`./uniffi_bindgen/src/bindings/`](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_bindgen/src/bindings):** This module turns a
      [`ComponentInterface`](https://docs.rs/uniffi_bindgen/latest/uniffi_bindgen/interface/struct.ComponentInterface.html) into *foreign-language bindings*,
      the code that can load the FFI layer exposed by Rust and expose it as a
      higher-level API in a target language. There is a sub-module for each internally supported language.

- **`./uniffi_meta`**([docs.rs](https://docs.rs/uniffi_meta/latest), [source](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_meta)):
The types used to represent the metadata used to describe the `ComponentInterface` used to generate the Rust scaffolding and the foreign bindings.

- **`./uniffi_udl`**([docs.rs](https://docs.rs/uniffi_udl/latest), [source](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_udl)) :
The parsing of UDL files, turning them into `uniffi_meta` types.

- **`./uniffi_build`**([docs.rs](https://docs.rs/uniffi_build/latest), [source](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_build)):
A small hook to run `uniffi-bindgen` from the `build.rs` script
of a UniFFI component, in order to automatically generate the Rust scaffolding as part of its build process.

- **`./uniffi_macros`**([docs.rs](https://docs.rs/uniffi_macros/latest), [source](https://github.com/mozilla/uniffi-rs/tree/main/uniffi_macros)):
Contains the proc_macro support, which does much of the heavy-lifting for defining the Rust FFI.

- **[`./examples`](https://github.com/mozilla/uniffi-rs/tree/main/examples):**
Contains code examples demonstrating the capabilites and code generation process.

- **[`./fixtures`](https://github.com/mozilla/uniffi-rs/tree/main/fixtures):**
Our test suite.
