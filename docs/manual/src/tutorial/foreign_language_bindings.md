# Foreign-language bindings

As stated in the [Overview](../Overview.md), this library and tutorial does not cover *how* to ship a Rust library on mobile, but how to generate bindings for it, so this section will only cover that.

First, make sure you have installed all the [prerequisites](./Prerequisites.md) - in particular:
  - `uniffi-bindgen` to generate the Rust scaffolding (or alternatively, understanding how to run it from the source tree)
  - `uniffi-bindgen-[language]` for each language you want to generate bindings for.

## Examples:
  - **Kotlin**: Running `uniffi-bindgen-kotlin src/math.udl` will generate `math.kt` at `src/uniffi/math/`
  - **Swift**: Running `uniffi-bindgen-swift src/math.udl` will generate `match.swift`, `mathFFI.h`, and `mathFFI.modulemap` at
    `src/uniffi/math`.
  - **Python**: Running `uniffi-bindgen-python src/math.udl` will generate `math.py` at `src/uniffi/math/`
  - **Ruby**: Running `uniffi-bindgen-python src/math.udl` will generate `math.rb` at `src/uniffi/math/`

## Standard vs external bindings generators

The 4 languages listed above represent the standard UniFFI bindings generators.  They are developed inside the `uniffi-rs` repo and are
supported by the UniFFI team.  However, you can also create external crates to generate bindings for other languages.  These crates
generall depend on `uniffi-bindgen` to handle some common tasks, and extend to with bindings generation code and a user-facing CLI.

## Standard CLI arguments

All of the standard bindgen tools support the same arguments:

 - `--help`: Print help text
 - `--out-dir=[path]`: Path to the directory to generate code in
 - `--config=[path]`: Path to the `uniffi.toml` config file
 - `--no-format`: Disable code formatting
 - `[udl_file]` (positional arg): path to the UDL file

`uniffi-bindgen-kotlin`, `uniffi-bindgen-python`, and `uniffi-bindgen-ruby` also support `--stdout` which causes the output to be
printed to `STDOUT` rather than written to a file.

External bindings generations should support the same arguments, if it makes sense for their language.  However, they are free to
change the arguments they accept as they see fit.
