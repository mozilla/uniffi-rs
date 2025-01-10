# Doc annotation in proc-macro fails

Reported in [#2134](https://github.com/mozilla/uniffi-rs/issues/2134):

In v0.27 parsing of `#[doc]` annotations fail to parse if they contain anything but plain strings.

```
error: Cannot parse doc attribute
 --> fixtures/regressions/doc-annotation-fails/src/lib.rs:5:1
  |
5 | #[doc = std::concat!("A", "has a", "B")]
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```
