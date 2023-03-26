
This crate exists to test version mismatches between the scaffolding and the bindings.  This
happens when:

  - The scaffolding and bindings are generated with incompatible versions of
    UniFFI (`UNIFFI_CONTRACT_VERSION` differs).
  - The scaffolding and bindings are generated from different UDL files
  - The scaffolding and bindings are generated from different proc-macro
    wrapped code.

The crate has scripts which trigger version mismatches and run bindings scripts
in order to verify the output.

Ideally this would be a trybuild-style test that checks the output.  However,
that's tricky for a variety of reasons so we just have the hacky scripts.
