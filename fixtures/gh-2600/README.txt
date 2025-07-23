Tests github.com/mozilla/uniffi-rs/issues/2600, where objects are freed too early.
For unknown reasons, the only known type this happens with is `__m256i`.
