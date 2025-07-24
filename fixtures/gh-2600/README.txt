Tests github.com/mozilla/uniffi-rs/issues/2600, where objects are freed too early.
This happens when the alignment is 32 (256 bits).
Lower alignments don't trigger the bug, higher alignments probably do.
`align(64)` also triggered the bug when tested.
Large alignments likely change the layout of `Arc<T>` so the `usize` refcount is in a different place compared to `Arc<std::ffi::void>`.
