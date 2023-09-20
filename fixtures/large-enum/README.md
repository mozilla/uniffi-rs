This crate tests the issue discussed in [#1631](https://github.com/mozilla/uniffi-rs/issues/1631)
where the `uniffi::Enum` proc-macro would fail for large enums. There are no tests for this because
we only need to test that the code that derives the `uniffi::Enum` trait compiles.