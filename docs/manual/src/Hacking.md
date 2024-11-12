# Hacking on UniFFI code

If you're interested in hacking on UniFFI code, please do!
We're always open to outside contributions.
This page contains some tips for doing this.

## Testing bindings code with `test-bindgen`

Use the `test-bindgen` tool to test out changes to the foreign bindings generation code.

- `cd` to a fixture/example directory
- Run `cargo run -p test-bindgen <language> print`.  This will print out the generated code for the current crate.
- Run `cargo run -p test-bindgen <language> save-diff`.  This will save a copy of the generated code for later steps.
- Make changes to the code in `uniffi-bindgen/src/bindings/<language>`
- Run `cargo run -p test-bindgen <language> diff`.  This will print out a diff of the generated code from your last `save-diff` run.

