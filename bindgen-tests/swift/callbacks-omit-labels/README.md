# Regression test for [issue 1312](https://github.com/mozilla/uniffi-rs/issues/1312)

We didn't apply the same `omit_argument_labels` configuration to callbacks,
leading to a compilation error.

We replicate the reported buggy code and turn that into a full blown test now.
